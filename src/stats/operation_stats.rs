use super::{
    call_chain::{
        caching_process_label, get_call_chain, CChainStats, CChainStatsKey, CChainStatsValue,
        CallChain,
    },
    error_stats::{get_cchain_error_information, get_span_error_information},
    file::OperationStatsJson,
    proc_oper_stats::{ProcOperStats, ProcOperStatsValue},
};
use crate::{processed::Span, utils};

#[derive(Debug, Default)]
pub struct OperationStats {
    /// The Operation either inbound (when this process acts as a server) or outbound (when this process is the client that initiates the request)
    pub operation: ProcOperStats,
    /// num_traces is used, as the name says, to find how many traces use this value.
    /// The other call values below can be inflated in case each trace can call a operation many times.
    pub num_traces: usize,
    /// number of inbound calls to this Process/Operation. The process is incoded in the key this value belongs to
    pub num_received_calls: usize,
    /// number of outbound calls from this Process/Operation to other proceses
    pub num_outbound_calls: usize,
    /// Unknown calls can come from partially malformed (corrupted traces), where either the sending or the receiving side of a call are missing (these are two records in a complete trace!)
    pub num_unknown_calls: usize,
    /// The statistics over all call-chains that lead to this Process/Operation
    pub call_chain: CChainStats,
}

impl OperationStats {
    pub fn new() -> Self {
        Default::default()
    }
}

impl From<OperationStatsJson> for OperationStats {
    fn from(stj: OperationStatsJson) -> Self {
        Self {
            num_traces: stj.num_traces,
            num_received_calls: stj.num_received_calls,
            num_outbound_calls: stj.num_outbound_calls,
            num_unknown_calls: stj.num_unknown_calls,
            operation: stj.method,
            call_chain: stj.call_chain.into_iter().collect(),
        }
    }
}

impl OperationStats {
    // The update_stat closure is the actual update operation
    // This closure is later applied to the newly inserted record for this process, or is used to update an existing record,
    // such that both processes share exactly the same code.
    pub fn update(
        &mut self,
        idx: usize,
        span: &Span,
        spans: &Vec<Span>,
        caching_process: &Vec<String>,
    ) {
        match &span.span_kind {
            Some(kind) => match &kind[..] {
                "server" => self.num_received_calls += 1,
                "client" => self.num_outbound_calls += 1,
                _ => self.num_unknown_calls += 1,
            },
            None => self.num_unknown_calls += 1,
        }

        let duration_micros = span.duration_micros;
        let start_dt_micros = span.start_dt.timestamp_micros();
        let (http_not_ok_vec, error_logs_vec) = get_span_error_information(span);

        let update_proc_oper_value = |oper_stat_val: &mut ProcOperStatsValue| {
            oper_stat_val.count += 1;
            oper_stat_val.start_dt_micros.push(start_dt_micros);
            oper_stat_val.duration_micros.push(duration_micros);
            oper_stat_val.num_not_http_ok += if http_not_ok_vec.is_empty() { 0 } else { 1 };
            oper_stat_val.num_with_error_logs += if error_logs_vec.is_empty() { 0 } else { 1 };
            oper_stat_val
                .http_not_ok_codes
                .add_items(http_not_ok_vec.clone());
            oper_stat_val.error_logs.add_items(error_logs_vec.clone());
        };
        // add a count per method
        let method = &span.operation_name;
        self.operation
            .0
            .entry(method.to_owned())
            .and_modify(|oper_stat| update_proc_oper_value(oper_stat))
            .or_insert_with(|| {
                let mut oper_stat = ProcOperStatsValue::default();
                update_proc_oper_value(&mut oper_stat);
                oper_stat
            });

        // // add a count per method_including-cached
        let call_chain = get_call_chain(idx, spans);
        let (http_not_ok_vec, error_logs_vec) = get_cchain_error_information(idx, spans);
        let caching_process = caching_process_label(caching_process, &call_chain);

        // add call-chain stats
        let depth = call_chain.len();
        let looped = get_duplicates(&call_chain);
        let is_leaf = span.is_leaf;
        let rooted = span.rooted;
        let cc_not_http_ok = if http_not_ok_vec.is_empty() { 0 } else { 1 };
        let cc_with_error_log = if error_logs_vec.is_empty() { 0 } else { 1 };

        let ps_key = CChainStatsKey {
            call_chain,
            caching_process,
            is_leaf,
        };

        let update_ps_val = |ps: &mut CChainStatsValue| {
            ps.count += 1;
            ps.start_dt_micros.push(start_dt_micros);
            ps.duration_micros.push(duration_micros);
            ps.cc_not_http_ok += cc_not_http_ok;
            ps.cc_with_error_logs += cc_with_error_log;
            ps.http_not_ok.add_items(http_not_ok_vec.clone()); // clone needed as otherwise this will be an FnOnce while rust thinks it is used twicecargo
            ps.error_logs.add_items(error_logs_vec.clone());
        };
        self.call_chain
            .entry(ps_key)
            // next part could be made more dry via an update-closure
            .and_modify(|ps| update_ps_val(ps))
            .or_insert_with(|| {
                let mut ps = CChainStatsValue::new(depth, looped, rooted);
                update_ps_val(&mut ps);
                ps
            });
    }

    /// header for report_stats_line output in ';'-separated csv-format
    pub fn report_stats_line_header_str() -> &'static str {
        "Process; Num_received_calls; Num_outbound_calls; Num_unknown_calls; Perc_received_calls; Perc_outbound_calls; Perc_unknown_calls"
    }

    /// reports the statistics for a single line in ';'-separated csv-format
    pub fn report_stats_line(&self, process: &str, num_traces: f64) -> String {
        let freq_rc = self.num_received_calls as f64 / num_traces;
        let freq_oc = self.num_outbound_calls as f64 / num_traces;
        let freq_uc = self.num_outbound_calls as f64 / num_traces;
        format!(
            "{process}; {}; {}; {}; {}; {}; {}",
            self.num_received_calls,
            self.num_outbound_calls,
            self.num_unknown_calls,
            utils::format_float(freq_rc),
            utils::format_float(freq_oc),
            utils::format_float(freq_uc)
        )
    }
}

/// get all values that appear more than once in the list of strings, while being none-adjacent.
/// TODO: this is part of a procedure to detect loops, which is not completely correct I guess (in case spans are missing)
fn get_duplicates(names: &CallChain) -> Vec<String> {
    let mut duplicates = Vec::new();
    for idx in 0..names.len() {
        let proc = &names[idx].process;
        let mut j = 0;
        loop {
            if j >= duplicates.len() {
                break;
            }
            if duplicates[j] == *proc {
                break;
            }
            j += 1;
        }
        if j < duplicates.len() {
            continue;
        }
        //  nme does not exist in duplicates yet, so find it in names
        let mut j = idx + 2; // Step by 2 as we want to prevent matching sub-sequent GET calls
        loop {
            if j >= names.len() || names[j].process == *proc {
                break;
            }
            j += 1;
        }
        if j < names.len() {
            duplicates.push(proc.to_owned());
        }
    }
    duplicates
}

# jaeger_stats
Parse Jaeger-json files in order to collect trace statistics

## How to run an analysis
You can run the tool on a single Jaeger-trace via the command:

```
trace_analysis  <data_folder>  -c <data_folder>/CallChain
```

Here data_folder can be an absolute or a relative path, however the expansion of '~'  to a home-folder is not supported. The path-encoding needs to match the conventions of your system (Windows or Linux/Unix/Mac). 

The tool will analyse all read all json-file in the folder (assuming these are valid Jaeger-trace files) and will process these files and compute statistics. Each json file can contains one or more traces. Output will be generated in the next folders:
* <data_folder>/Traces: contains a single file for each trace. This file is name <trace_id>.txt and contains fairly concise textual representation of the jaeger-trace
* <data_folder>/Stats: contains file with the statistics over traces. The most important one is 'Stats/cummulative_trace_stats.csv' which contains statistics over all traces. However, you will also see a number of other files such as 'Stats/gateway_POST__services_orders_update.csv' which contains the statistics over the subset of traces originating from the end-point 'gateway/POST:/services/orders/update/'.
* <data_folder>/CallChain: This folder contains a text-files such as for example 'Stats/gateway_POST__services_orders_update.cchain' which contains a list of all call-chains that originate at the API-gateway endpoint 'gateway/POST:/services/orders/update/'. So each line in this cchain-file represents a unique series of process (microservices) that appears in the input-traces. These Cchain files give an impression of the complexity of the processing, and these files also serve a purpose in the correction of incomplete traces, which is the topic of a separate section.
* report.txt: a structured log-file showing a summary and detail information on the analysis process.

Traces will be deduplicated before analysis based on the 'trace_id'  so if the folder contains files that overlap in traces they contain this overlap is removed.

When you run the commandd with flag `--help` you see:
```
$ trace_analysis --help
Parsing and analyzing Jaeger traces

Usage: trace_analysis [OPTIONS] <INPUT>

Arguments:
  <INPUT>  

Options:
      --caching-process <CACHING_PROCESS>      
  -c, --call-chain-folder <CALL_CHAIN_FOLDER>  [default: /home/ceesvk/CallChain/]
  -t, --timezone-minutes <TIMEZONE_MINUTES>    [default: 120]
  -f, --comma-float                            
  -h, --help                                   Print help
  -V, --version                                Print version
```
The options are:
* --caching-process: a comma separated list of processes that apply caching of results. This information os relevant as the call-chains that contain these services are called less often as the downstream data migh be cached. If you know the cache-hit-rates you are able to correct the leaf nodes to compute the expected number of calls when the cache is turned off (or flushed). It is also possible to acctually compute the cache-hit ratios by comparing the traffic on the 'path/cached_service' vs 'path/cached_service *LEAF*', where the version marked with  '*LEAF*' are the the calls that do not have any downstream processing This can happens for example when a cache-hits removes the need for downstream analysis. However, this be care-ful this also occures if the service does not do down-stream calls for other reasons, such as incorrect or empty parameters.
* --call-chain-folder: The folder containing files used to correct incomplete call-chains 
* --timezone-minutes: The offset in minutes for the current timezone relative to UTC. The default value is 120 minutes which corresponds to AMS-timezone
* -- comma-float: In CSV files floating point values are using a comma as separator instead of the '.' to allow the file to be read in an Excel. The default value is 'true' (using )

## Contents of the files with statistics
The statistics files, such as 'Stats/cummulative_trace_stats.csv' use the ';' as the column separator. This file falls apart in four sections:
1. Generic information such as, the list of trace_ids, the start_times of these traces and the average duration of these process
2. Process-information: Lists all processes (services) in the call-chain and shows the number of inbound and outbound on this service. However it does not contain any details on the opertion being called)
3. Process/operation: List the statistics like call-frequency, average time, max time, etc.. for each process/service
4. Call-chain: List statistics for the full-call chain and also shows whether a service is a leaf-node or contains further downstream calls. Please note that the execution-time of a service/operation includes the execution time of all downstream calls performed. However, if you all heavy lifting is done in leaf-nodes the sum of the average time of the Leaf-nodes should come close to the average trace duration.

## Correction of call-chains



## How to build trace_analysis
The tool is include in the examples folder and can be build via the command:

```
cargo build trace_analysis
```

The 'trace_analysis' executable can be found in 'target/debug/examples/trace_analysis'.

In case you need to process a large volume of traces you might aim for the more performant 'release' build (which also drops some run-time checks).  To build a release version use:

```
cargo build --release trace_analysis
```

The 'trace_analysis' executable can be found in 'target/release/examples/trace_analysis'.


You can also install the tool via 

```
cargo install --release trace_analysis
```

On linux this will deploy a release version of 'trace_analysis' in the folder '$HOME/.cargo/bin/' which is assumed to be included in your path. 
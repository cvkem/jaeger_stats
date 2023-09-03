# jaeger_stats
The jaeger_stats is a library project focussed on handling and analyzing jaeger-traces. Jaeger-traces provide very detailled information. This is very useful for a detailled issue analysis. Hoevever this can also be a very useful source of information on how processes run in a complex microservices landscape and to gain insights how the landscape and the pressure on the individual service evolve over time.

This Jaeger_stats also contains a few tools (executables) build on top of the library to show-case how the tooling can be used, or even to use the tooling.

## How to run an analysis
You can run the tool on a single Jaeger-trace via the command:

```
trace_analysis  <data_folder> 
```

Here data_folder can be an absolute or a relative path, however the expansion of '~'  to a home-folder is not supported. The path-encoding needs to match the conventions of your system (Windows or Linux/Unix/Mac). 

The tool will analyse all read all json-file in the folder (assuming these are valid Jaeger-trace files) and will process these files and compute statistics. Each json file can contains one or more traces. Output will be generated in the next folders:
* <data_folder>/Traces: contains a single file for each trace. This file is name <trace_id>.txt and contains fairly concise textual representation of the jaeger-trace
* <data_folder>/Stats: contains file with the statistics over traces. The most important one is 'Stats/cummulative_trace_stats.csv' which contains statistics over all traces. However, you will also see a number of other files such as 'Stats/gateway_POST__services_orders_update.csv' which contains the statistics over the subset of traces originating from the end-point 'gateway/POST:/services/orders/update/'. Next to each of the .csv files we will save a .json file with the same based that contains the full dataset (csv-files are a sub-set for reading in excel. The full files are used for later post-processing, for example by the 'stitch' tool)
* <data_folder>/CallChain: This folder contains a text-files such as for example 'Stats/gateway_POST__services_orders_update.cchain' which contains a list of all call-chains that originate at the API-gateway endpoint 'gateway/POST:/services/orders/update/'. So each line in this cchain-file represents a unique series of process (microservices) that appears in the input-traces. These Cchain files give an impression of the complexity of the processing, and these files also serve a purpose in the correction of incomplete traces, which is the topic of a separate section. Via configuration it is possble to move this 'CallChain' folder to another location such that this folder can be shared between different data_folders.
* report.txt: a structured log-file showing a summary and detail information on the analysis process. 

Traces will be deduplicated before analysis based on the 'trace_id'  so if the folder contains files that overlap in traces they contain this overlap is removed.

When you run the trace_analysis with flag `--help` you see:
```
$ trace_analysis --help
Parsing and analyzing Jaeger traces

Usage: trace_analysis [OPTIONS] <INPUT>

Arguments:
  <INPUT>  

Options:
      --caching-process <CACHING_PROCESS>
          
  -c, --call-chain-folder <CALL_CHAIN_FOLDER>
          The default source for call-chain information is a sub-folder'CallChain' located in the current folder [default: CallChain/]
  -z, --timezone-minutes <TIMEZONE_MINUTES>
          [default: 120]
  -f, --comma-float
          
  -t, --trace-output
          
  -o, --output-ext <OUTPUT_EXT>
          The output-extension determines the output-types are 'json' and 'bincode' (which is also used as the file-extension) [default: json]
  -h, --help
          Print help
  -V, --version
          Print version
```
The options are:
* --caching-process: a comma separated list of processes that apply caching of results. This information os relevant as the call-chains that contain these services are called less often as the downstream data migh be cached. If you know the cache-hit-rates you are able to correct the leaf nodes to compute the expected number of calls when the cache is turned off (or flushed). It is also possible to acctually compute the cache-hit ratios by comparing the traffic on the 'path/cached_service' vs 'path/cached_service *LEAF*', where the version marked with  '*LEAF*' are the the calls that do not have any downstream processing This can happens for example when a cache-hits removes the need for downstream analysis. However, this be care-ful this also occures if the service does not do down-stream calls for other reasons, such as incorrect or empty parameters.
* --call-chain-folder (-c): The folder containing files used to correct incomplete call-chains
* --timezone-minutes (-z): The offset in minutes for the current timezone relative to UTC. The default value is 120 minutes which corresponds to AMS-timezone
* -- comma-float (-f): In CSV files floating point values are using a comma as separator instead of the '.' to allow the file to be read in an Excel. The default value is 'true'
* --trace_output (-t): a boolean to signal whether the '<data_folder>/Traces' should be filled with traces. The default is 'false' as these traces can be volumeous data.
* --output-ext: If the output-ext is set to 'json' (default) which means that the output is written to a json-file. The alternative is 'bincode'. Writing 'bincode' files is faster, but the format is not human readible.

## Contents of the files with statistics
The statistics files, such as 'Stats/cummulative_trace_stats.csv' use the ';' as the column separator. This file falls apart in four sections:
1. Generic information such as, the list of trace_ids, the start_times of these traces and the average duration of these process
2. Process-information: Lists all processes (services) in the call-chain and shows the number of inbound and outbound on this service. However it does not contain any details on the opertion being called)
3. Process/operation: List the statistics like call-frequency, average time, max time, etc.. for each process/service
4. Call-chain: List statistics for the full-call chain and also shows whether a service is a leaf-node or contains further downstream calls. Please note that the execution-time of a service/operation includes the execution time of all downstream calls performed. However, if you all heavy lifting is done in leaf-nodes the sum of the average time of the Leaf-nodes should come close to the average trace duration.

## Correction of call-chains
Jaeger tracing spans are send over UDP, which is a protocol that does not give strong delivery guarantees. So occasionally a span might be lost which results in an incomplete trace, and thus broken call-chains in the trace. This is where the weird '-c' option pops up as seen in the previous example: `trace_analysis  <data_folder>  -c <data_folder>/CallChain`. Here the CallChain produced by the first run of the tool (only showing complete chains) will be used in the subsequent runs of the tool to correct incomplete call-chains for missing spans. However, the preferred option is to set up a separate folder to contain the call-chains, refer the '--call-chain-folder' or '-c' to this folder.

The call-chain corrections are only applied:
* to traces that do miss some of the spans.
* to call-chains that do not exist in the call-chain-file for the end-point of the current trace
* in case the call-chain can be matched exactly on the tail of 1 other call-chain. So if more than one match exist the correction will not be applied.


## Correction of operations (path parameters)
Path parameters might wreak havoc on our analysis as path parameters make each URL unique while we are looking for averages over a number of invocations Therefore the system does correction on the URL's to extract the parameters, for example an order number and replaces that with a symbolic value '{ORDER}'. However, these replacements are currently hardcoded and we need to take some steps to make this configurable.

## Computation of the rates (request/second)
If data is provided in a large batches it is possible to compute the rate from the data. However, we do not want to assume that all files with traces fall in the same time-period. Therefore we compute frequencies by computing times between subsequent calls and dropping the num_files largest intervals, as these might corresponds to gaps inbetween files. Based on this time the rate is computed as a frequency by the formula f=1/T  where T is the duration in seconds between subsequent calls.


## Extracting Jaeger JSON data
In the Jaeger web-based front end it is possible to make a selection of traces. After these traces have been returned you have two methods to extract the JSON files:
1. Click on a single trace and in the right-top of the page select Download as 'JSON'.
2. Open the developers tools and navigate to the network-tab. Now fire the request:
   1. Navigate to the response page. It might take some time to download the data and to transform and pretty-print the JSON. Select the full response and copy-paste it to a file
   2. Right-click on the response and select 'Copy Curl-URL' (for your system). Paste this URL in a console and redirect the output to a file.
Using method 2.1 you can get approximately 1000 traces in a batch. The batch will be available as pretty-printed JSON in UTF8.

Method 2.2 allows you to select 1000 traces or more. However, the output a single line of raw json (not-pretty-printed) and the file is encoded in UTF-16-LE with BOM. The 'trace_analysis' can handle these files and will do an in-memory conversion to UTF8 before processing. Beware that this is a non-streaming conversion so the full file is in memory twice.


## Using stitch-tool to merges results of different runs 
The stitch tool is used to take a series of trace_analysis outputs and stitch them together to a single time-series analysis. The inputs are defined in a file 'input.stitch'.

The collected (time-series) output is written to a file 'stitch.csv' (default) which can easily read into Microsof Excel.
The output contains (fine-grained) metrics-data as a time-series for all:
* process/operation combinations
* call-chains (call paths), with is basically an additional level of details as most process/operations can be reached over multiple call-chains.
Each time-series is amended with a linear regression analysis for that time-series.

Next to the detailled output a file is generated that shows the anomalies (outliers) that have been detected.


When you run the 'stitch' with flag `--help` you see:

```
$ stitch -h`
Stitching results of different runs of trace_analysis into a single CSV for visualization in Excel

Usage: stitch [OPTIONS]

Options:
  -s, --stitch-list <STITCH_LIST>                      [default: input.stitch]
  -o, --output <OUTPUT>                                [default: stitched.csv]
  -a, --anomalies <ANOMALIES>                          [default: anomalies.csv]
  -c, --comma-float                                    
  -d, --drop-count <DROP_COUNT>                        [default: 0]
      --scaled-slope-bound <SCALED_SLOPE_BOUND>        [default: 0.05]
      --st-num-points <ST_NUM_POINTS>                  [default: 5]
      --scaled-st-slope-bound <SCALED_ST_SLOPE_BOUND>  [default: 0.05]
      --l1-dev-bound <L1_DEV_BOUND>                    [default: 2]
  -h, --help                                           Print help
  -V, --version                                        Print version
```

The options are:
* --stitch_list: a file that shows the paths for all result.json files that need to be stitched together. All text after a '#' is considered comments. Empty lines are ignored (including lines that start with a comment) and lines that start with a % will show up as an empty column in the analysis (used to temporarily exclude a missing file or file containing outliers). Text after the '%' is ignored. All relative paths in the stitch-list are expected to start in the folder that contains the 'input.stitch' file, such that you can move the complete folder of the 'input.stitch' to a different location.   
* --output: The output-file in CSV-format that contains the data stitched together. Each column in this file represents a single input-file from 'input.stitch'. Each statistic is a separate line and the second column represents the name of the statistic. 
* -- comma-float: In CSV files floating point values are using a comma as separator instead of the '.' to allow the file to be read in an Excel. The default value is 'true' (using )


An example of an input-file ('input.stitch') is:
```
#  comment line: this line is full ignored
/home/ceesvk/jaeger/batch/Stats/cummulative_trace_stats.json       # an absolute path
../../jaeger/get_order/Stats/cummulative_trace_stats.json    # a relative path
% ../../jaeger/post_order/Stats/cummulative_trace_stats.json  # This line is showing up as an empty column due to the % in front

# yet another comment (empty line above is ignored)
```

Beware that ALL files in the 'input.stitch' should exist and should be valid input files, otherwise the 'stitch' program will terminate with no output. 

## Extracting traces with the show_traces tool
When extracting datasets via Curl or other tools the Jaeger system returns up to 1000 traces in a single file. This file is in UTF-16-LE encoding instead of UTF-8 and is a JSON-file in a compact (minimized) format. Thus it is difficult to read these files, or to extract data out of them. For this purpose we proved the show_traces tool. It reads all jaeger-traces in a folder and then outputs these traces in a single file per trace in the folder 'Jaeger'. If are only interested in a few specific files you can provide the trace-ids of these files as a comma-separate list.

When you run the show_traces with flag `--help` you see:
```
Show the Jaeger-traces, or a selection of jaeger-traces, as Pretty-printed JSON in UTF-8 format

Usage: show_traces [OPTIONS] <INPUT>

Arguments:
  <INPUT>  

Options:
  -t, --trace-ids <TRACE_IDS>                The default sources is the current folder [default: ]
  -z, --timezone-minutes <TIMEZONE_MINUTES>  [default: 120]
  -h, --help                                 Print help
  -V, --version                              Print version
```

## How to install the Jaeger_stats tools
the Jaeger_stats tooling is deployed to pypi.org as a Python project via an automated Github CI/CD pipeline.
Thus the tools can be installed easily on Windows, Mac and Linux via the next command:

```
pip install jaeger_stats
```
If you need pre-releases of the tool you need to use:
```
pip install --pre --force-reinstall jaeger_stats
```


## How to build trace_analysis (in Rust)

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



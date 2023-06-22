# jaeger_stats
Parse Jaeger-json files in order to collect trace statistics

## How to run
You can run the tool on a single Jaeger-trace via the command:

```
trace_analysis  ABC.json
```

where ABC.json is a Jaeger trace in json format.

This will analyze the file and produce two new files in the same folder:

* ABC.txt:  contains some statistics followed by a fairly concise textual representation of the jaeger-trace
* ABC.csv: contains statistics computed over the Jaeger-trace which can serve as input for further modelling and analysis.


You can also run the command for a folder:

```
trace_analysis  folder/sub-folder/
```

In that case each of the json-files in the folder will be analyzed and the two files as mentioned above a produced. On top of that an additional file is produced with the 'cummulative_trace_stats.csv' that contains the cummulative statistics over all traces.


## Process to build tool
The tool is include in the examples folder and can be build via the command:

```
cargo build trace_analysis
```

or if you need a bit more speed 

```
cargo build --release trace_analysis
```

The 'trace_analysis' file is created in the 'target' folder.


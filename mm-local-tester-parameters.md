# Basic Usage

```
java -jar <tester-jar-file.jar> [-seed valueOrRange] [-exec yourSolution]
```

- `-seed S` Run a single test case S.
- `-seed S1,S2` Run a range of seeds, between S1 and S2, inclusive.
- `-seed S+D` Run a range of D seeds, starting from S, equivalent to `-seed S,(S+D-1)`
- `-seed {S1,S2,…,Sn}` Run a list of specified seeds, using curly braces.
- `-seed S1*K` or `S1,S2*K` or `S+D*K` or `{S1,S2,…,Sn}*K` Run K times a single seed, a range or a list of seeds.
- `-exec "<command>"` If your compiled solution is an executable file, the command will be the full path to it. For example, `"C:\TC\solution.exe"` or `"~/topcoder/solution"`. In case your compiled solution is to be run with the help of an interpreter, for example, if your program is in Java, the command will be something like `"java -cp C:\TopCoder Solution"`.

## Examples

```
java -jar <tester-jar-file.jar> -seed 1,10 -exec "tc/mysolution"
```

Run the program named "mysolution" (an executable located in folder "tc") 10 times, using all seeds between 1 and 10 (one at a time).

```
java -jar <tester-jar-file.jar> -seed 758+200 -exec "java -cp bin MySol"
```

Run a Java solution (a compiled class file located in folder "bin"), using seeds between 758 and 957, inclusive.

All parameters are case-insensitive, and there are also shorter alternative parameter names:

- `-seed` is equivalent to `-sd`
- `-exec` is equivalent to `-ex`

## Visual Options

**`-novis` (or `-nv`)**

For problems with a visual representation, this option disables the visualization.

**`-size` (or `-sz`)**

Controls the size of the visualization area. It is the number of pixels used by each logical unit in the viewer content (e.g. the size of each cell in a grid). A value of zero (`-sz 0`) can be used to fit the visualizer window into the screen area. Note that the visualizer window is now resizable.

**`-windowPos x,y[width,height]` (or `-wp`)**

Allows setting the visualizer's initial window position (x, y) and optionally its dimensions (width, height). It can be useful when placing it side-by-side with the other open windows (e.g. code editor) or when working with more than one screen. All values are in pixels, using screen coordinates.

**`-screen N` (or `-sc`)**

A simpler way to place the visualizer's window in a specific (Nth) screen, when working with multiple monitors. The default behavior is to place the visualizer window in the primary (N=1) screen. To place it in a secondary screen, use `-screen 2`. When combined with `-windowPos` parameter, the window position coordinates are relative to the specified screen.

**`-saveVis [folder]` (or `-sv`)**

Allows saving the visualizer content into an image file, after each screen update. A sequence of PNG images will be generated in the specified folder (or in the current folder, if the folder parameter is omitted), using the filename "seed-SSSSS.png", where SSSSS is a sequential number. Note that the dimensions of the image will match the dimensions of the visualizer in the screen, so it may be useful to adjust the desired dimensions beforehand, using `-size` or `-windowPos` parameters. It is necessary that the visualization itself is enabled (`-noVis` parameter should not be used together with this option). This option will save all the visualizer's window content, which includes a central main panel and the information panel on the right side of the window. To save only the central panel content, use together with the option `-is 0`, as described below.

**`-infoScale scale` (or `-is`)**

Allows increase/decrease the font used in the information panel (right side of visualizer). It is a percentage, so 100 means no change in the displayed font size. `-infoScale 75` would use a smaller font, and `-is 150` a larger one. If `-is 0` is used, the information panel is not displayed at all, which may be useful if the competitor wants to see only the visualizer's main (central) content, possibly together with `-saveVis` parameter.

**`-delay` (or `-dl`)**

For problems with animation in which intermediate states are displayed, this option controls the time interval (in milliseconds) between each update of the visualizer. A value of zero (`-delay 0`) can be used to go straight to the final state of the solution.

**`-pause` (or `-ps`)**

For problems with animation this option allows starting the visualizer in a paused state. Space key can be used to toggle the pause state on/off when the visualizer is running. Other keys advance a single step.

## Other Options

**`-debug` (or `-db`)**

Enable debug messages from the tester.

**`-noOutput` (or `-no`)**

Disable printing messages from your solution.

**`-printRuntime` (or `-pr`)**

Enable printing the run time, together with the score.

**`-timeLimit <milliseconds>` (or `-tl`)**

Enable timeout control. For example `-timeLimit 5000` will enforce a run time limit of 5000 ms (i.e. 5 seconds).

**`-threads <numThreads>` (or `-th`)**

Enable running multiple parallel tests when running multiple seeds with a single command. For example `-threads 4` will run up to four tests at the same time. The default is using a single thread. This option may be useful to run a batch of tests faster. Note that when using more than one thread, there will be some level of concurrence, depending on your system and the number of threads used, which will affect the solution performance. Also note that the actual number of threads used is limited to the number of virtual CPUs available.

**`-saveSolInput [<folder>]` (or `-si`)**

Save the input provided to your solution in the specified folder using the filename `<seed>.in`. If the folder is omitted, the current folder is used. All save options overwrite existing files in the specified folder (if they exist). They automatically create the specified folder if it doesn't exist.

Example: `-seed 1,100 -saveSolInput input`: Save in the folder "input" files named 1.in, 2.in, ..., 100.in, containing the first 100 test cases. Later, your program could be called directly redirecting one of these files to its standard input: `mysolution < 1.in`

**`-saveSolOutput [<folder>]` (or `-so`)**

Save the standard output produced by your solution in the specified folder, using the filename `<seed>.out`. If the folder is omitted, the current folder is used. Files created with this option can be used later as input to the tester (see option `-loadSolOutput` below).

**`-saveSolError [<folder>]` (or `-se`)**

Save the error output produced by your solution in the specified folder, using the filename `<seed>.err`. If the folder is omitted, the current folder is used.

**`-saveAll [<folder>]` (or `-sa`)**

Combine all three saving options (`-saveSolInput`, `-saveSolOutput` and `-saveSolError`), saving in the specified folder the input provided to your solution (`<seed>.in`), the output produced by your solution (`<seed>.out`) and the error output produced by your solution (`<seed>.err`). If the folder is omitted, the current folder is used.

**`-loadSolOutput [<folder>]` (or `-lo`)**

Instead of calling an executable file, if this parameter is specified the tester will read the solution from the specified folder. If the folder is omitted, the current folder is used.

It can't be used together with `-exec` parameter. It can be useful to evaluate a solution generated outside the tester (even manually). If you run your solution directly and redirect its standard output to a file (`mysolution < inputs/1.in > outputs/1.out`), the generated file can be evaluated using `-seed 1 -loadSolOutput outputs`

**`-bests <bestControlFile>` (or `-bs`)**

Save the best score seen so far for each seed in the specified file. This feature uses a simple control and it is not intended to replace more advanced local testers. When enabled, it will print the best value seen for that seed, before the current execution, and then will update the file if the new value is better.

**`-saveScores <scoresFile>` (or `-ss`)**

Save to the specified file the seeds and scores from the current execution. It can be useful when running multiple seeds, especially using multiple threads (in which the standard out may be shuffled). The generated file lists seeds and their scores, keeping seeds in ascending order, using the same format of the bests file (`-bests` option). This option will override the content of the specified file if it exists.

**`-noAntialiasing` (or `-na`)**

Disable anti-aliasing when rendering visualizer content. It can be useful to increase the rendering speed in slow machines, although it will also reduce the rendering quality.

## Contest-specific Parameter Overriding

**`-<param> <valueOrRange>`**

This can be used to override test case parameters.

For example, in MM 116 there were N grids (from 5 to 100).

It would be possible to use `-N 100` to force all cases to have 100 grids.

Or `-N 5,15` for a small number of grids (randomly selected between 5 and 15 inclusive).

## Execution Summary

When running multiple seeds, the tester will automatically print a summary including the number of tests executed, number of failed tests, average and maximum running time:

```
Seeds: 1001 to 1500

Executed Cases: 500

Failed Cases: 0

Avg. Run Time: 9476 ms

Max. Run Time: 9834 ms
```

If `-bests` option is enabled, it will also include in this summary the number of bests improved, tied, normalized previous best score and normalized score of the current execution:

```
Improved Bests: 23

Tied Cases: 36

Prev. Bests Score: 98.42355

Current Score: 82.30097
```

In the table below there is an example of how these scores are calculated, considering the goal is to minimize the score. If the goal was to maximize, the idea would be the same, but the relative score for each test case would be "score / best" (maximize), instead of "best / score" (minimize).

Screenshot 2020-07-02 11:10:12

The default behavior is printing the summary, but it can be suppressed with the parameter `-noSummary` (or `-ns`).

## Compiling From Source

To compile the tester from its source, use the command line javac compiler:

```
javac <TesterClassName>.java
```

Note that the tester has a contest-specific source file and other common files from the package `com.topcoder.marathon`. The compiler will automatically find and compile these other common files if they are in their correct subfolder `com/topcoder/marathon`.

To create a runnable jar from the compiled classes run:

```
jar cvfe <TesterJarName>.jar <TesterClassName> .
```

Competitors who want to make changes to provided local tester for any reason may find useful information in this [document](writing-a-marathon-tester.md) (originally target to problem writers).

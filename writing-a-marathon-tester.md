# Class/method to implement

A marathon tester with no visual representation (tester only) should extend MarathonTester abstract base class, which already implements many common features.

Three abstract methods must be implemented:

- **`boolean isMaximize()`** — Return true if the goal of this problem is to maximize the score, or false otherwise (minimize).
- **`void generate()`** — Generate the test case.
- **`double run()`** — Run this test case, calling the solution (if defined) and return the calculated raw score.

For problems with a visual representation but no animation, i.e. only a single (final) state is shown, the class to be extended is MarathonVis (which extends MarathonTester). There is a single additional method to be implemented:

- **`void paintContent(Graphics2D g)`** — Responsible for painting the content of the visualizer. Detailed information is available in the "Visualization and Painting" section.

Finally, for problems with animations, i.e. where intermediate states are displayed, the base class to be extended is MarathonAnimatedVis. It has no abstract methods.

mtester 1

## Random generator

The random number generator is already created and initialized to use the specified seed by the base class. It is available as a class variable named `rnd`, to be used inside `generate()` method (and occasionally inside `run()`, in the case of interactive problems, in which the tester needs to take actions based on solution output).

There are convenience methods to get a random number (integer or double) given a range. Please use them whenever possible.
Example: if some variable N is randomly selected between [10,50](inclusive), just use:

```java
int N = randomInt(10, 50);
```

## Command line parameters

All command line parameters are parsed by the MarathonController class. Most of the code to handle common parameter options are also implemented by the controller or base tester classes.

Problem specific parameters can be obtained from the `parameters` class variable.
Included in the problem specific parameters are main test cases random variables, that could be overridden by competitors, using command line parameters, to test specific types of cases. Example: if the problem has a grid of size S, between 10 and 50 inclusive the code could be something like:

```java
int S = randomInt(10, 50);
if (parameters.isDefined("S")) S = randomInt(parameters.getIntRange("S"));
```

So after implementing the regular code to generate, it is recommended to add this feature for the random parameters that would make sense to force a specific value or range. It should be just one extra line of code for each parameter.
Note that even if a single value is specified by the competitor (using `-S 50`), `randomInt(parameters.getIntRange())` will work as expected. So the tester code should always be implemented in the way shown in the example, which allows both ranges and single values. Similar methods are available for doubles.

It is also possible to enforce the limits, so the user specified range is trimmed to be contained in the regular range. That is important when out of range values will certainly lead to error, so they would be useless. Same example, with range enforcement. In this case, if `-S 40,100` is used, the range would be [40,50], i.e. the intersection between valid range ([10,50]) and user specified range ([40,100]).

```java
   int S = randomInt(minSize, maxSize); //Using constants is better
   if (parameters.isDefined("S")) {
     S = randomInt(parameters.getIntRange("S"), minSize, maxSize);
   }
```

## Time Control

Time control is mostly implemented by MarathonTester base class. This includes timeout control and time measure. Concrete implementations should call `startTime()` right before waiting for output produced by the solution. And call `stopTime()` right after reading all solution output.
These functions can be called multiple times, for interactive problems. The time control will accumulate the elapsed time. It always needs to be called in pairs (start/stop), without starting again the timer before stopping it.
It is typically called inside the `callSolution()` method, as in the following example:

```java
private String[] callSolution() throws Exception {
  writeLine(size);
  writeLine(numColors);
  flush();
  if (!isReadActive()) return null;

  startTime(); // <-- Start here

  int n = readLineToInt(-1);
  if (n < 0) {
    setErrorMessage("Invalid number of moves: " + getLastLineRead());
    return null;
  }

  String[] ret = new String[n];
  for (int i = 0; i < n; i++) {
    ret[i] = readLine();
  }

  stopTime(); // <-- Stop when solution finished returning everything
  return ret;
}
```

## Input/Output

Output (writing data that will be read by the solution) and input (reading solution's answer) should use methods implemented by MarathonTester base class. These functions take care of where to read from (solution or an input file) and where to write to (solution and/or an output file).

As can be seen in the example code above, to write:

- `writeLine()` writes a string or a single integer.
- `flush()` when writing is done.

For reading:

- `readLine()` reads a full line to a String
- `readLineToInt()` reads a full line and converts the content into a single integer. An exception will be thrown if it is not an integer value.
- `readLineToInt(int invalid)` same as before, but returns the specified invalid value when it is unable to parse an integer value.
- `readLineToIntArr()` a convenience method that reads a full line, with space-separated integers, and returns an integer array.
- `getLastLineRead()` can be used to get the content of the last read line, usually to create an error message.
- `isReadActive()` before starting reading (and after writing all possible input), the tester should call this function to check if there is any active read source. Otherwise it should return immediately. This will happen when the tester is called without `-exec` and `-loadSolOutput` options.

## Main Method

The tester should run the `main()` method, just calling the MarathonController:

```java
public static void main(String[] args) {
  new MarathonController()
    .run(args);
}
```

When building the tester JAR, this should be the execution entry point.

## Visualization and Painting

The base class MarathonVis takes care of things like creating a frame, finding its correct size, and adjusting its layout.
Using a standard way of displaying information makes it possible that visualizers work in any screen resolution and allows smooth window content resizing, without extra effort in the contest-specific part of the code. The visualizer is divided in two parts: the content panel and the information panel, as shown below.

panel

The content is painted by the contest-specific tester, while the info panel is painted by the MarathonVis, using information provided by the specific tester.

### Initialization

In the beginning of the `run()` method, the tester implementation should call three methods that will provide information to MarathonVis about the desired layout:

```java
    if (hasVis()) {
      setInfoMaxDimension(20, 12);
      setContentRect(0, 0, w, h);
      setDefaultSize(30);
    }
```

The first one (`setInfoMaxDimension`) sets the number of columns and rows of text that the information panel will contain. These numbers don't need to be exact, just an approximate upper bound that will reserve enough space for the displayed information.

The second one (`setContentRect`) sets the desired logical coordinates of the content area. Later, all painting will use these coordinates, as explained below.

The last one, which is optional, will set the default size in pixels of one content unit. It is the default value equivalent to the size specified in the command line option `-sz`. If it is not set, it will use the standard value 0, which means fit to screen.

It is important to note that the tester can call `hasVis()` method to check if visualization is enabled (i.e. if it wasn't disabled by the command line option `-novis`), and avoid running unnecessary parts of the code.

### Information Panel

As mentioned, the information panel is painted by the base class. The tester implementation should call the following functions to provide the information to be displayed:

- `addInfo(Object key, Object value)` this will display "key: value" in the info panel. If it is called multiple times, it updates the respective value, without adding a new row of information.
- `addInfo(Object key)` displays a centralized "key" (useful for section titles).
- `addInfoBreak()` skips one line of text, to separate sections of information.

In the following example, it is possible to see the calls made to build the information panel shown:

seed

In the case of an animated visualizer, after processing each step of the solution, fields like steps and score can be updated by simply calling `addInfo()` again, with the new values, only for fields that changed. In the previous example, an update after each step would have:

```java
addInfo("Steps", currStep + " / " + numSteps);
addInfo("Score", getScore());
addInfo(Color.red, 12);
addInfo(Color.blue, 9);
addInfo(Color.green, 11);
```

These functions receive Objects as parameters. The default behavior is just to convert the content to a String, but it is possible to create different visual representations for other types of objects. Currently, there is support for objects of the Color class, which are displayed as small squares filled with the Color object value.

### Content Panel

The tester should implement the abstract method `paintContent()`. As the desired coordinates were already specified, inside this method there is no need to worry about borders, window dimension or the information panel. All painting should be made inside the contentRect previously set.

If the problem deals with a w x h grid and contentRect was set to (0, 0, w, h), some examples of painting:

Fill with blue the left half of the grid area:

```java
    Rectangle2D rc = new Rectangle2D.Double(0, 0, w / 2, h);
    g.setColor(Color.blue);
    g.fill(rc);
```

Fill with red the cell grid with coordinates x = 2, y = 3 (i.e. the third column and fourth row):

```java
    Rectangle2D rc = new Rectangle2D.Double(2, 3, 1, 1);
    g.setColor(Color.red);
    g.fill(rc);
```

Draw a line connecting the center of grid cells (1,3) and (8,0):

```java
    Line2D line = new Rectangle2D.Double(1.5, 3.5, 8.5, 0.5);
    g.setStroke(new BasicStroke(0.03 f, BasicStroke.CAP_ROUND,
      BasicStroke.JOIN_ROUND));
    g.draw(line);
```

Note that line thickness (stroke) also uses the current coordinate system, so it should be specified as a proportion of a grid unit (in this example, 3% of the size of a grid cell).

### Update

After the initial setup and whenever the tester wants to update the visualization content, it should call the `update()` method. This method already checks whether the visualization is enabled, so it can be called without checking `hasVis()`.

### Lock

As painting is asynchronous, the tester should make all updates to state class variables inside a synchronized block, on the object `updateLock`, already created for this purpose, to avoid painting intermediate (and possibly invalid) states in the middle of a transition.

### Paint Text in the Content Panel [NEW]

As explained above, a coordinate transformation is applied to the Graphics2D object, making painting content easier (no need to worry about borders, offsets, size parameter, the actual window dimensions and so on).
Although positioning text inside the visualizer content is simple (and follows the same idea used for any graphical component), setting the font size can be tricky.
Two methods were added to MarathonVis class, to make this task much simpler:

- **`adjustFont`**: receives the font that will be used, but instead of specifying its size directly, it will inform the maximum (larger) string that will be painted and a rectangle that this string should fit. For example, if number with up to 3 digits will be displayed inside each grid cell, the call should be something like:

```java
adjustFont(g, Font.SANS_SERIF, Font.PLAIN,
"000", new Rectangle2D.Double(0, 0, 0.6, 0.6));
```

Note that the rectangle size used is 0.6 (instead of 1.0) to leave a margin. This method should be called inside `paintContent()`, only once for each group of strings that will use the same font.

- **`drawString`**: instead of receiving x,y coordinates, it will receive a rectangle that will "contain" the string. The text will be drawn centralized (both horizontally and vertically) in the specified rectangle.

An example of paintContent implementation, that will produce the following grid:

Screenshot 2020-06-26 16:46:53

```java
    protected void paintContent(Graphics2D g) {
      adjustFont(g, Font.SANS_SERIF, Font.PLAIN,
        "000", new Rectangle2D.Double(0, 0, 0.6, 0.6));
      for (int y = 0; y < size; y++) {
        for (int x = 0; x < size; x++) {
          Rectangle2D rc = new Rectangle2D.Double(x, y, 1, 1);
          g.setColor(color[y][x]);
          g.fill(rc);
          g.setColor(Color.black);
          g.draw(rc);
          drawString(g, String.valueOf(x + y * size), rc);
        }
      }
      g.setStroke(new BasicStroke(0.04 f, BasicStroke.CAP_ROUND,
        BasicStroke.JOIN_ROUND));
      g.setColor(Color.black);
      for (int i = 0; i <= size; i++) {
        g.draw(new Line2D.Double(0, i, size, i));
        g.draw(new Line2D.Double(i, 0, i, size));
      }
    }
```

## Animated Visualizers

For problems with animation, in which intermediate states are displayed, the base class MarathonAnimatedVis takes care of delays and pausing.

All that tester implementation needs to do is call `updateDelay()` whenever it wants to update the visualizer. It will automatically pause or delay depending on the current state/configuration.

Inside the loop that processes each step of the solution, the tester should call `hasDelay()` (usually together with `hasVis()`) to check if it is necessary to repeat calculations (like updating the current score) after each step. The `hasDelay()` function returns false if delay is zero, which means no intermediate states should be updated.

In the beginning of the `run()` method, tester implementations can call `setDefaultDelay(int delay)` to set the default delay between each step.

## Paint Performance [NEW]

To help checking if visualizer content rendering is fast enough, especially when there are many intermediate states to be drawn, it is possible to use the command line parameter `-paintInfo` (or `-pi`). It will print (console standard output) how many times visualizer content was painted and the average time of each paint (in milliseconds).

## A Complete Example

Here is a complete (and working) example of an animated visualizer, with additional comments in important places.

It is a very simple problem called ShortestPath. There is a grid, and two distinct randomly chosen cells on it. The goal is to move from the start cell to the target cell. The score, which should be minimized, is the squared number of steps the solution takes to reach the target.

Screenshot 2020-06-26 16:50:54

```java
import java.awt.*;
import java.awt.geom.*;
import com.topcoder.marathon.*;

public class ShortestPathTester extends MarathonAnimatedVis {
  //Ranges
  private static final int minSize = 8, maxSize = 50;

  //Constraints
  private static final int maxSteps = 10000;

  //Inputs
  private int size;
  private int x0, y0, x1, y1;

  //State control
  private int xp, yp;
  private int step;

  //Generate the test case (override an abstract method of MarathonTester) 
  protected void generate() {
    size = randomInt(minSize, maxSize);

    //Special cases for seed 1 and 2
    if (seed == 1) {
      size = minSize;
    } else if (seed == 2) {
      size = maxSize;
    }

    //User defined parameters
    if (parameters.isDefined("N")) size = randomInt(parameters.getIntRange("N"),
      minSize, maxSize);

    //Generate distinct start and target positions
    x0 = randomInt(0, size - 1);
    y0 = randomInt(0, size - 1);
    do {
      x1 = randomInt(0, size - 1);
      y1 = randomInt(0, size - 1);
    } while (x0 == y0 && x1 == y1);

    if (debug) {
      System.out.println("N = " + size);
      System.out.println("start = " + x0 + "," + y0);
      System.out.println("target = " + x1 + "," + y1);
    }
  }

  //Return false as the goal here is to minimize the score 
  //(override an abstract method of MarathonTester) 
  protected boolean isMaximize() {
    return false;
  }

  //Run the solution (override an abstract method of MarathonTester) 
  protected double run() throws Exception {
    //Initialization
    init();
    int score = getErrorScore();

    //Call the solution and check its return
    String[] ret = callSolution();
    if (ret == null) {
      if (!isReadActive()) return getErrorScore();
      return fatalError();
    }
    if (ret.length > maxSteps) return fatalError("Your solution exceed the maximum number of" +
      " steps " + maxSteps + " steps (returned " + ret.length + ")");

    //Initial update
    if (hasVis() && hasDelay()) {
      addInfo("Steps", 0 + " / " + ret.length);
      updateDelay();
    }

    //Loop through all steps
    while (step < ret.length) {
      //Updates to state class variables should be done inside a synchronized block
      synchronized(updateLock) {
        String s = ret[step++];
        if (debug) System.out.println("Step " + step + ": " + s);
        if (s.length() != 1) return fatalError("Step " + step + " does not contain" +
          "exactly one character:" + s);

        //Process one step, checking invalid moves and out of bounds
        char c = s.charAt(0);
        int x = xp;
        int y = yp;
        if (c == 'U') y--;
        else if (c == 'D') y++;
        else if (c == 'R') x++;
        else if (c == 'L') x--;
        else return fatalError("Step " + step + " contains an invalid move: " + c);
        if (y < 0 || x < 0 || y >= size || x >= size) {
          return fatalError("Step " + step + " moved out of bounds.");
        }

        //Update current position
        xp = x;
        yp = y;
      }

      //Update if necessary (hasVis and (hasDelay or it is the last step))
      if (hasVis() && (hasDelay() || step == ret.length)) {
        score = step * step;
        if (step == ret.length && (xp != x1 || yp != y1)) score = getErrorScore();
        synchronized(updateLock) {
          addInfo("Steps", step + " / " + ret.length);
          addInfo("Score", score);
        }
        updateDelay();
      }
    }
    return score;
  }

  //Paint the content panel (override an abstract method of MarathonVis) 
  protected void paintContent(Graphics2D g) {
    //Fill grid background
    Rectangle2D rc = new Rectangle2D.Double(0, 0, size, size);
    g.setColor(Color.white);
    g.fill(rc);

    //Fill start and target cells
    rc = new Rectangle2D.Double(x0, y0, 1, 1);
    g.setColor(Color.red);
    g.fill(rc);
    rc = new Rectangle2D.Double(x1, y1, 1, 1);
    g.setColor(Color.blue);
    g.fill(rc);

    //Draw grid
    g.setStroke(new BasicStroke(0.04 f, BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
    g.setColor(Color.black);
    for (int i = 0; i <= size; i++) {
      g.draw(new Line2D.Double(0, i, size, i));
      g.draw(new Line2D.Double(i, 0, i, size));
    }

    //Current position
    Ellipse2D e = new Ellipse2D.Double(xp + 0.1, yp + 0.1, 0.8, 0.8);
    g.setColor(new Color(0, 0, 0));
    g.fill(e);
    g.setColor(new Color(255, 255, 255));
    g.draw(e);
  }

  private void init() {
    //Initial position
    xp = x0;
    yp = y0;

    if (hasVis()) {
      //Sets the maximum dimension of information panel (number of rows and columns of text)
      setInfoMaxDimension(18, 9);

      //Set desired coordinates of the content to a squared grid
      setContentRect(0, 0, size, size);

      //Default size (in pixels) of grid's cell
      setDefaultSize(30);

      //Default delay (in milliseconds) between visualizer updates
      setDefaultDelay(100);

      //Create information panel content
      addInfo("Seed", seed);
      addInfoBreak();
      addInfo("Size N", size);
      addInfo("Start", "(" + x0 + ", " + y0 + ")");
      addInfo("Target", "(" + x1 + ", " + y1 + ")");
      addInfoBreak();

      //Steps and score will be updated later, while processing solution return
      addInfo("Steps", "-");
      addInfoBreak();
      addInfo("Score", "-");

      //Call update when done
      update();
    }
  }

  private String[] callSolution() throws Exception {
    //Write input data
    writeLine(size);
    writeLine(x0);
    writeLine(y0);
    writeLine(x1);
    writeLine(y1);

    //Flush when writing is done
    flush();

    //Check is there is any active read (a running solution or a solution file), 
    //otherwise just return.
    if (!isReadActive()) return null;

    //Start to measure time spent by the solution
    startTime();

    //Read from the solution (process or a file) the number of lines
    int n = readLineToInt(-1);
    if (n < 0) {
      setErrorMessage("Invalid number of steps: " + getLastLineRead());
      return null;
    }

    //Read each line
    String[] ret = new String[n];
    for (int i = 0; i < ret.length; i++) {
      ret[i] = readLine();
    }

    //Stop to measure time
    stopTime();

    return ret;
  }

  //Main method should create a MarathonController and call run passing command line arguments
  public static void main(String[] args) {
    new MarathonController()
      .run(args);
  }
}
```

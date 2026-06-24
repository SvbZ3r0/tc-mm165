import java.awt.*;
import java.awt.geom.*;
import java.util.*;
import java.util.List;
import javax.imageio.ImageIO;
import java.awt.event.KeyEvent;

import com.topcoder.marathon.*;

public class BattleshipsTester extends MarathonAnimatedVis {
    // parameter ranges
    private static final int minN = 8, maxN = 20; // grid size range
    private static int minS, maxS;    // num ships range, based on N
    private static final double minP = 0.1, maxP = 1.0; // multiplier range

    // Inputs
    private int N;    // grid size
    private int S;    // num ships
    private int L;    // maximum ship length, always N/2
    private double P; // multiplier

    // Cell Constants
    private static final int Empty=0;
    private static final int NearShip=-1;
    private static final char Hit='*';
    private static final char Killed='x';
    private static final char Unseen='~';
    private static final char Miss='.';

    // Behaviour Constants
    //"count" : number of ship cells, "binary": 0 if there are no ship cells, otherwise 1
    private static final String scanReturn = "count";    
    //"sqrt" = P*sqrt(area), "log" = P*log2(area+1)
    private static final String scanScore = "log";
    private static final boolean useSpacesBetweenShips = false;
    //IMPORTANT: change to true when running on the server
    private static final boolean useServer = false;   

    // Graphics
    private Image[] shipImagesH;
    private Image[] shipImagesV;    
    private Image fireImage;
    private Image missImage;
    private static final Color seaColour = new Color(81, 153, 186);
    private static final Color shipColour = Color.lightGray;
    private static final Color fireColour = Color.orange;
    private static final Color missColour = Color.pink;

    // State Control
    private int[][] startGrid;
    private char[][] userGrid;
    private int[] shipLengths;    
    private Ship[] ships;
    private int shipsLeft = 0;
    private int turn = 0;
    private int shots = 0;
    private int scans = 0;
    private int hits = 0;
    private int kills = 0;
    private int misses = 0;
    private int selectedR = -1;
    private int selectedC = -1;
    private int selectedR2 = -1;
    private int selectedC2 = -1;    
    private int selectedCount = -1;    
    private double score = 0;


    protected void generate()
    {
      N = randomInt(minN, maxN);
      minS = N/2;
      maxS = 3*N/2;
      S = randomInt(minS, maxS);
      P = randomDouble(minP, maxP);

      // Special cases
      if (seed == 1)
      {
        N = minN;
        S = N/2;
      }
      else if (seed == 2)
      {
        N = maxN;
        S = 3*N/2;
      }

      // User defined parameters
      if (parameters.isDefined("N"))
      {
        N = randomInt(parameters.getIntRange("N"), minN, maxN);
        minS = N/2;
        maxS = 3*N/2;        
      }
      if (parameters.isDefined("S")) S = randomInt(parameters.getIntRange("S"), minS, maxS);
      if (parameters.isDefined("P")) P = randomDouble(parameters.getDoubleRange("P"), minP, maxP);

      L=N/2;
      shipsLeft=S;
      userGrid=new char[N][N];
      for (int r=0; r<N; r++)
        for (int c=0; c<N; c++)
          userGrid[r][c]=Unseen;

      placeShips();

      if (debug)
      {
        System.out.println("Grid size, N = " + N);
        System.out.println("Number of ships, S = " + S);
        System.out.println("Maximum ship length, L = " + L);      
        System.out.println("Multiplier, P = " + P);
        System.out.println("Ships:");
        for (int i=1; i<=L; i++)
          System.out.println(shipLengths[i]+" ship(s) of length "+i);
        System.out.println("Start Grid:");
        printGrid(startGrid);
        System.out.println();
      }
    }


    protected void placeShips()
    {
      int iter=1;
loop:
      for(; true; iter++)
      {
        startGrid=new int[N][N];
        ships=new Ship[S+1];
        shipLengths=new int[L+1];

        for (int id=1; id<=S; id++)   //ship id
        {
          int length=randomInt(1,L);

          //find candidate locations for this ship
          List<Ship> candidates=new ArrayList<Ship>();     
          for (int dir=0; dir<=1; dir++)
            for (int r=0; r<N; r++)
              for (int c=0; c<N; c++)
              {
                if (dir==0 && c+length-1>=N) continue;
                if (dir==1 && r+length-1>=N) continue;

                //can we place the ship?
                boolean ok=true;
                for (int k=0; k<length; k++)
                  if ((dir==0 && startGrid[r][c+k]!=Empty) || (dir==1 && startGrid[r+k][c]!=Empty))
                  {
                    ok=false;
                    break;
                  }

                if (!ok) continue;    //cannot place ship

                Ship s=new Ship(id,r,c,dir,length);
                candidates.add(s);
              }


          if (candidates.size()==0) continue loop;    //no valid placements, so try again

          //choose a random ship placement
          int q=randomInt(0,candidates.size()-1);
          Ship s=candidates.get(q);

          //now actually place the ship
          ships[id]=s;
          shipLengths[s.length]++;
          int r=s.r;
          int c=s.c;
          int dir=s.dir;
          for (int k=0; k<length; k++)
          {
            startGrid[r][c]=id;

            //horizontal
            if (dir==0)
            {
              //mark spaces that we cannot use (if selected)   
              if (useSpacesBetweenShips)
              {
                if (k==0 && inGrid(r,c-1))  startGrid[r][c-1]=NearShip;
                if (inGrid(r-1,c))          startGrid[r-1][c]=NearShip;
                if (inGrid(r+1,c))          startGrid[r+1][c]=NearShip;
                if (k==length-1 && inGrid(r,c+1))  startGrid[r][c+1]=NearShip;
              }
              c++;
            }
            //vertical
            else
            {
              //mark spaces that we cannot use (if selected)   
              if (useSpacesBetweenShips)
              {
                if (k==0 && inGrid(r-1,c))  startGrid[r-1][c]=NearShip;
                if (inGrid(r,c-1))          startGrid[r][c-1]=NearShip;
                if (inGrid(r,c+1))          startGrid[r][c+1]=NearShip;
                if (k==length-1 && inGrid(r+1,c))  startGrid[r+1][c]=NearShip;      
              }
              r++;          
            }
          }
        }

        break;    //found a valid grid
      }   

      if (debug) System.out.println("attempts to make grid = "+iter);   
    }


    protected void printGrid(int[][] g)
    {
      for (int r = 0; r < N; r++) {
        for (int c = 0; c < N; c++) {
          String a = "";
          if (g[r][c] == Empty || g[r][c] == NearShip) a = ".";
          else if (g[r][c] > 0 && g[r][c] <= 9) a = "" + g[r][c];
          else a = ""+(char)('A' + (g[r][c]-10));
          System.out.print(a);
        }
        System.out.println();
      }
    }


    protected boolean isMaximize() {
        return false;
    }

    protected double run() throws Exception {
        init();

        if (parameters.isDefined("manual")) {
            setDefaultDelay(0);
            updateState();
            return 0;
        } else return runAuto();
    }

    protected double runAuto() throws Exception {
        double score = callSolution();
        if (score < 0) {
            if (!isReadActive()) return getErrorScore();
            return fatalError();
        }
        return score;
    }

    protected void timeout() {
        addInfo("Time", getRunTime());
        updateState(); 
    }

    private double callSolution() throws Exception {
        writeLine("" + N);
        writeLine("" + S);
        writeLine("" + L);
        writeLine("" + P);
        for (int i=1; i<=L; i++)
          writeLine(""+shipLengths[i]);

        flush();
        if (!isReadActive()) return -1;

        score=0;
        updateState();        

        for (turn=1; shipsLeft>0; turn++)
        {
          try {
          // read solution output
          startTime();
          String line=readLine();
          stopTime();
          String[] temp=line.split(" ");

          if (temp.length==3)
          {
            if (!temp[0].equals("SHOOT"))
              return fatalError("Invalid format! Should be SHOOT R C");

            int r=Integer.parseInt(temp[1]);
            int c=Integer.parseInt(temp[2]);
            if (!inGrid(r,c))
              return fatalError("Location ("+r+","+c+") is out of bounds!");              

            if (userGrid[r][c]!=Unseen)
              return fatalError("You already shot at ("+r+","+c+")!");              

            //perform shot
            shots++;
            if (startGrid[r][c]>=1)
            {
              Ship s=ships[startGrid[r][c]];
              s.lives--;
              hits++;
              score+=1;     //hit cost

              if (s.lives==0)
              {
                kills++;
                shipsLeft--;
                shipLengths[s.length]--;

                //mark ship cells as killed
                int r2=s.r;
                int c2=s.c;
                for (int i=0; i<s.length; i++)
                {
                  userGrid[r2][c2]=Killed;
                  if (s.dir==0) c2++;
                  else r2++;
                }

                writeLine("KILL");
              }
              else
              {
                userGrid[r][c]=Hit;
                writeLine("HIT");
              }
            }
            else
            {
              misses++;
              score+=1;
              userGrid[r][c]=Miss;
              writeLine("MISS");
            }

            //print elapsed time
            writeLine(""+getRunTime());    
            flush();    
          }
          else if (temp.length==5)
          {
            if (!temp[0].equals("SCAN"))
              return fatalError("Invalid format! Should be SCAN R1 C1 R2 C2");

            int r1=Integer.parseInt(temp[1]);
            int c1=Integer.parseInt(temp[2]);
            int r2=Integer.parseInt(temp[3]);
            int c2=Integer.parseInt(temp[4]);            
            if (!inGrid(r1,c1))
              return fatalError("Location ("+r1+","+c1+") is out of bounds!");             
            if (!inGrid(r2,c2))
              return fatalError("Location ("+r2+","+c2+") is out of bounds!");    


            //perform scan
            scans++;
            int area=(Math.abs(r1-r2)+1)*(Math.abs(c1-c2)+1);

            if (scanScore.equals("sqrt"))
              score+=P*Math.sqrt(area);
            else if (scanScore.equals("log"))
              score+=P*Math.log(N*N+2-area)/Math.log(2);
            else
              return fatalError("scanScore not recognised: "+scanScore);

            int count=0;
            for (int r=Math.min(r1,r2); r<=Math.max(r1,r2); r++)
              for (int c=Math.min(c1,c2); c<=Math.max(c1,c2); c++)              
                if (startGrid[r][c]>=1)
                  count++;

            if (scanReturn.equals("binary")) count=(count > 0 ? 1 : 0);

            writeLine(""+count);

            //print elapsed time
            writeLine(""+getRunTime());  
            flush();   

            selectedR=r1;
            selectedC=c1;                       
            selectedR2=r2;
            selectedC2=c2;            
            selectedCount=count;
            if (!parameters.isDefined("noanimate")) updateState();                    
          }
          else
          {
            return fatalError("Invalid format! "+line);            
          }          
          if (!parameters.isDefined("noanimate")) updateState();

        } catch (Exception e) {
            if (debug) System.out.println(e.toString());
            return fatalError("Cannot parse your output");
        }}

      if (parameters.isDefined("noanimate")) updateState();                

      return score;
    }

    protected boolean inGrid(int r, int c) {
      return r >= 0 && r < N && c >= 0 && c < N;
    }

    protected void updateState() {
      if (hasVis()) {
          synchronized (updateLock) {
              addInfo("Turns", turn);
              addInfo("Shots", shots);
              addInfo("Scans", scans);
              addInfo("Hits", hits);
              addInfo("Kills", kills);
              addInfo("Misses", misses);
              addInfo("Time", getRunTime());
              addInfo("Score", String.format("%.3f", score));

              addInfo("Ships Left", shipsLeft); 
              for (int i=1; i<=L; i++)
                addInfo("Length "+i,shipLengths[i]);
          }
          updateDelay();
      }
    }


    protected void contentClicked(double x, double y, int mouseButton, int clickCount)
    {
      if (!parameters.isDefined("manual")) return;

      if (shipsLeft==0) return;   //already found everything

      int r = (int) Math.floor(y);
      int c = (int) Math.floor(x);

      if (!inGrid(r, c)) return;    // outside of the grid

      //shoot with left click
      if (mouseButton == java.awt.event.MouseEvent.BUTTON1)
      {       
        if (userGrid[r][c]!=Unseen) return;   //already shot here

        shots++;        
        if (startGrid[r][c]>=1)
        {
          Ship s=ships[startGrid[r][c]];
          s.lives--;
          hits++;
          score+=1;   //hit cost

          if (s.lives==0)
          {
            kills++;
            shipsLeft--;
            shipLengths[s.length]--;

            //mark ship cells as killed
            int r2=s.r;
            int c2=s.c;
            for (int i=0; i<s.length; i++)
            {
              userGrid[r2][c2]=Killed;
              if (s.dir==0) c2++;
              else r2++;
            }
          }
          else
          {
            userGrid[r][c]=Hit;
          }
        }
        else
        {
          misses++;
          score+=1;
          userGrid[r][c]=Miss;
        }
        updateState();
        return;
      }

      //scan with right click
      if (mouseButton == java.awt.event.MouseEvent.BUTTON3)      
      {
        //first click select cell
        if (selectedR==-1 && selectedR2==-1)
        {
          selectedR=r;
          selectedC=c;
          updateState();
        }
        //select second cell and perform the scan
        else if (selectedR!=-1 && selectedR2==-1)
        {
          selectedR2=r;
          selectedC2=c;           
          int r1=selectedR;
          int c1=selectedC;
          int r2=r;
          int c2=c;

          scans++;
          int area=(Math.abs(r1-r2)+1)*(Math.abs(c1-c2)+1);

          if (scanScore.equals("sqrt"))
            score+=P*Math.sqrt(area);
          else if (scanScore.equals("log"))
            score+=P*Math.log(N*N+2-area)/Math.log(2);

          int count=0;
          for (r=Math.min(r1,r2); r<=Math.max(r1,r2); r++)
            for (c=Math.min(c1,c2); c<=Math.max(c1,c2); c++)              
              if (startGrid[r][c]>=1)
                count++;

          if (scanReturn.equals("binary")) count=(count > 0 ? 1 : 0);      

          selectedCount=count;
          updateState();                      
        }
        return;
      }      
    }


    //NOTE: all paths are drawn starting from the tip and going clockwise
    protected void paintContent(Graphics2D g)
    {
      g.setColor(seaColour);
      g.fillRect(0, 0, N, N);

      // draw ships
      // basic using shapes
      if (parameters.isDefined("noimages"))
      {
        g.setStroke(new BasicStroke(0.025f, BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
        for (int r = 0; r < N; r++)
          for (int c = 0; c < N; c++)
          {
            //ship cell
            if (userGrid[r][c] == Killed)
            {
              g.setColor(shipColour);

              Ship s=ships[startGrid[r][c]];

              //horizontal single
              if (r==s.r && c==s.c && s.length==1 && s.dir==0)             
              {
                GeneralPath gp = new GeneralPath();
                gp.moveTo(c, r+0.5);
                gp.lineTo(c+1.0/3, r);
                gp.lineTo(c+2.0/3, r);
                gp.lineTo(c+1, r+0.5);
                gp.lineTo(c+2.0/3, r+1);
                gp.lineTo(c+1.0/3, r+1);
                g.fill(gp);               
              }
              //vertical single
              else if (r==s.r && c==s.c && s.length==1 && s.dir==1)             
              {
                GeneralPath gp = new GeneralPath();
                gp.moveTo(c+0.5, r);
                gp.lineTo(c+1, r+1.0/3);
                gp.lineTo(c+1, r+2.0/3);
                gp.lineTo(c+0.5, r+1);
                gp.lineTo(c, r+2.0/3);
                gp.lineTo(c, r+1.0/3);
                g.fill(gp);               
              }            
              //left end
              else if (r==s.r && c==s.c && s.dir==0)
              {
                GeneralPath gp = new GeneralPath();
                gp.moveTo(c, r+0.5);
                gp.lineTo(c+0.5, r);
                gp.lineTo(c+1, r);
                gp.lineTo(c+1, r+1);
                gp.lineTo(c+0.5, r+1);
                g.fill(gp);              
              }
              //right end
              else if (r==s.r && c==s.c+s.length-1 && s.dir==0)
              {
                GeneralPath gp = new GeneralPath();
                gp.moveTo(c+1, r+0.5);
                gp.lineTo(c+0.5, r+1);
                gp.lineTo(c, r+1);
                gp.lineTo(c, r);
                gp.lineTo(c+0.5, r);
                g.fill(gp);                            
              }
              //top end
              else if (r==s.r && c==s.c && s.dir==1)
              {
                GeneralPath gp = new GeneralPath();
                gp.moveTo(c+0.5, r);
                gp.lineTo(c+1, r+0.5);
                gp.lineTo(c+1, r+1);
                gp.lineTo(c, r+1);
                gp.lineTo(c, r+0.5);
                g.fill(gp);                            
              }       
              //bottom end
              else if (r==s.r+s.length-1 && c==s.c && s.dir==1)
              {
                GeneralPath gp = new GeneralPath();
                gp.moveTo(c+0.5, r+1);
                gp.lineTo(c, r+0.5);
                gp.lineTo(c, r);
                gp.lineTo(c+1, r);
                gp.lineTo(c+1, r+0.5);
                g.fill(gp);                            
              }     
              //middle piece             
              else
              {
                g.fillRect(c, r, 1, 1);
              }
            }
            //fire
            if (userGrid[r][c] == Hit)
            {
              g.setColor(shipColour);
              g.fillRect(c, r, 1, 1);            
              g.setColor(fireColour);
              Ellipse2D t = new Ellipse2D.Double(c + 0.2, r + 0.2, 0.6, 0.6);
              g.fill(t);
            }          
            //miss
            if (userGrid[r][c] == Miss)
            {
              g.setColor(missColour);
              Ellipse2D t = new Ellipse2D.Double(c + 0.3, r + 0.3, 0.4, 0.4);
              g.fill(t);
            }                    
          }
      }
      //using images
      else
      {
        for (int r = 0; r < N; r++)
          for (int c = 0; c < N; c++)
          {
            //ship cell
            if (userGrid[r][c] == Killed || userGrid[r][c] == Hit)
            {
              Ship s=ships[startGrid[r][c]];            

              //horizontal
              if (s.dir==0)
              {
                Image I=shipImagesH[s.length];
                int imHeight=I.getHeight(null);
                int imWidth=I.getWidth(null);                 
                int num=c-s.c;
                int width=imWidth/s.length;
                int start=width*num;
                int end = (num==s.length-1 ? imWidth : width*(num+1));
                g.drawImage(I,c,r,c+1,r+1,start,0,end,imHeight,null);
              }
              //vertical
              else
              {
                Image I=shipImagesV[s.length];
                int imHeight=I.getHeight(null);
                int imWidth=I.getWidth(null);                                 
                int num=r-s.r;
                int height=imHeight/s.length;
                int start=height*num;
                int end = (num==s.length-1 ? imHeight : height*(num+1));
                g.drawImage(I,c,r,c+1,r+1,0,start,imWidth,end,null);
              }
            }
            //fire
            if (userGrid[r][c] == Hit)
            {  
              g.drawImage(fireImage,c,r,1,1,null);
            }          
            //miss
            if (userGrid[r][c] == Miss)
            {
              g.drawImage(missImage,c,r,1,1,null);
            }                    
          }        
      }

      // draw grid lines
      g.setColor(Color.black);
      g.setStroke(new BasicStroke(0.005f, BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
      for (int i = 0; i <= N; i++) {
          g.drawLine(i, 0, i, N);
          g.drawLine(0, i, N, i);
      }

      // draw selected cells
      // for scan
      if (selectedR != -1 && selectedR2 != -1)
      {
        int minR=Math.min(selectedR,selectedR2);
        int minC=Math.min(selectedC,selectedC2);
        int maxR=Math.max(selectedR,selectedR2);
        int maxC=Math.max(selectedC,selectedC2);                  

        if (scanReturn.equals("count"))
        {
          g.setColor(Color.black);
          g.setStroke(new BasicStroke(0.05f, BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
          g.drawRect(minC, minR, maxC-minC+1, maxR-minR+1);
          adjustFont(g, Font.SANS_SERIF, Font.PLAIN, String.valueOf(""+selectedCount), new Rectangle2D.Double(0, 0, 1, 1));
          drawString(g, ""+selectedCount, new Rectangle2D.Double(minC, minR, 1, 1));
        }
        else if (scanReturn.equals("binary"))
        {
          if (selectedCount==0) g.setColor(Color.red);
          else g.setColor(Color.green);

          g.setStroke(new BasicStroke(0.05f, BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
          g.drawRect(minC, minR, maxC-minC+1, maxR-minR+1);          
        }

        selectedR=-1;
        selectedC=-1;                       
        selectedR2=-1;
        selectedC2=-1;            
        selectedCount=-1;         
      }   
      else if (selectedR!=-1)
      {
        g.setColor(Color.black);
        g.setStroke(new BasicStroke(0.05f, BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));  
        g.drawRect(selectedC, selectedR, 1, 1);      
      }
    }

    Image loadImage(String name) {
      try{
        Image im=ImageIO.read(getClass().getResourceAsStream(name));
        return im;
      } catch (Exception e) { 
        return null;  
      }             
    }     

    private void init()
    {
      if (hasVis())
      {
        //load images
        shipImagesH=new Image[11];
        shipImagesV=new Image[11];        
        if (!parameters.isDefined("noimages") && !useServer)
        {
          for (int i=1; i<=10; i++)
          {
            shipImagesH[i] = loadImage("images/"+i+"H.png");
            shipImagesV[i] = loadImage("images/"+i+"V.png");
          }
          fireImage = loadImage("images/fire.png");
          missImage = loadImage("images/miss.png");
        }

        setDefaultDelay(100);

        setContentRect(0, 0, N, N);
        setInfoMaxDimension(20, 10);

        addInfo("Seed", seed);
        addInfo("size, N", N);
        addInfo("ships, S", S);
        addInfo("maxLength, L", L);
        addInfo("P", String.format("%.3f", P));

        addInfoBreak();
        addInfo("Ships Left", "-");
        for (int i=1; i<=L; i++)
          addInfo("Length "+i,shipLengths[i]);

        addInfoBreak();
        addInfo("Time", "-");
        addInfo("Turns", "-");
        addInfo("Shots", "-");
        addInfo("Scans", "-");
        addInfo("Hits", "-");
        addInfo("Misses", "-");
        addInfo("Kills", "-");
        addInfo("Score", String.format("%.3f", P));
        updateState(); 
      }
    }

    public static void main(String[] args) {
        new MarathonController().run(args);
    }

    public class Ship
    {
      int id;
      int r;
      int c;
      int dir;      //0=horizontal, 1=vertical
      int length;
      int lives;

      public Ship(int i, int r2, int c2, int d, int le)
      {
        id = i;
        r = r2;
        c = c2;
        dir = d;
        length = le;
        lives = length;
      }
    }
}
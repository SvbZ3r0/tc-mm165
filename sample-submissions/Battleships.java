import java.io.*;
import java.util.*;

public class Battleships
{
  public static void main(String[] args) throws Exception
  {           
    BufferedReader in = new BufferedReader(new InputStreamReader(System.in));    

    int N=Integer.parseInt(in.readLine());   
    int S=Integer.parseInt(in.readLine());   
    int L=Integer.parseInt(in.readLine());   
    double P=Double.parseDouble(in.readLine());   

    int[] shipLengths=new int[L+1];
    for (int i=1; i<=L; i++)
      shipLengths[i]=Integer.parseInt(in.readLine());

    boolean[][] seen=new boolean[N][N];

    int seed=42;
    int alive=S;
    while (alive>0)
    {
      seed=(seed*8009+104729)%(1<<16);
      int num=seed%(N*N);
      int r=num/N;
      int c=num%N;
      if (seen[r][c]) continue;

      seen[r][c]=true;
      System.out.println("SHOOT "+r+" "+c);
      System.out.flush();
      
      String result=in.readLine();
      if (result.equals("KILL")) alive--;
      
      int elapsedTime=Integer.parseInt(in.readLine());
    }
  }
}
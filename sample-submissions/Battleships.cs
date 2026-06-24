using System;
using System.Collections.Generic;

public class Battleships
{  
  public static void Main()
  {         
    int N = int.Parse(Console.ReadLine());
    int S = int.Parse(Console.ReadLine());
    int L = int.Parse(Console.ReadLine());
    double P = double.Parse(Console.ReadLine());

    int[] shipLengths=new int[L+1];
    for (int i=1; i<=L; i++)
      shipLengths[i]=int.Parse(Console.ReadLine());

    bool[,] seen=new bool[N,N];

    int seed=42;
    int alive=S;
    while (alive>0)
    {
      seed=(seed*8009+104729)%(1<<16);
      int num=seed%(N*N);
      int r=num/N;
      int c=num%N;
      if (seen[r,c]) continue;

      seen[r,c]=true;
      Console.WriteLine("SHOOT "+r+" "+c);
      Console.Out.Flush();
      
      string result=Console.ReadLine();
      if (result.Equals("KILL")) alive--;
      
      int elapsedTime=int.Parse(Console.ReadLine());
    }             
  }
}
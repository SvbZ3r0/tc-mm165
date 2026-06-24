// C++17
#include <iostream>
#include <list>
using namespace std;

int main() 
{
  int N, S, L;
  double P;
  cin >> N >> S >> L >> P;

  int shipLengths[L+1];
  for (int i=1; i<=L; i++)
    cin >> shipLengths[i];

  bool seen[N][N];
  for (int i=0; i<N; i++)
    for (int k=0; k<N; k++)
      seen[i][k]=false;

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
    cout << "SHOOT " << r << " " << c << endl;
    cout.flush();
   
    string result;
    cin >> result;
    if (result=="KILL") alive--;

    int elapsedTime;
    cin >> elapsedTime;
  }  
}

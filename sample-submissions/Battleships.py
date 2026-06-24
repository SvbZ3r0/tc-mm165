import sys

N = int(input())
S = int(input())
L = int(input())
P = float(input())

shipLengths=[0]*(L+1)
for i in range(1,L+1):
  shipLengths[i]=int(input())

seen = [[False for x in range(N)] for y in range(N)]

seed=42
alive=S
while alive>0:
  seed=(seed*8009+104729)%(1<<16)
  num=seed%(N*N)
  r=num//N
  c=num%N
  if seen[r][c]: continue

  seen[r][c]=True
  print("SHOOT "+str(r)+" "+str(c))
  sys.stdout.flush()  

  result=input()
  if result=="KILL": alive-=1

  elapsedTime=int(input())
// Scaling benchmark of the *scheduling strategy* (not Rust, not Android, not rendering).
// Model: N stored particles in components of fixed size S (chain collapse = whole component, instant).
// Each component collapses with per-tick prob q = 1-(1-p)^S; a collapsed component is respawned
// (population stays constant so the workload is steady). Strategies:
//   brute_particle : every tick, every particle rolls (u64 < threshold)          -- naive reading of the GDD
//   brute_component: every tick, one roll per component with q                   -- cheaper naive
//   event_driven   : min-heap of next-collapse ticks (geometric sampling)        -- ARCHITECTURE s3
// "Simulated" = particles whose state is evaluated by the tick loop. "Visible" = 0 (no rendering here).
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <math.h>
#include <time.h>
typedef uint64_t u64; typedef uint32_t u32;
static u64 s[4];
static inline u64 rotl(u64 x,int k){return (x<<k)|(x>>(64-k));}
static inline u64 nxt(void){u64 r=rotl(s[1]*5,7)*9,t=s[1]<<17;s[2]^=s[0];s[3]^=s[1];s[1]^=s[2];s[0]^=s[3];s[2]^=t;s[3]=rotl(s[3],45);return r;}
static void seed(u64 z){for(int i=0;i<4;i++){z+=0x9e3779b97f4a7c15ULL;u64 x=z;x=(x^(x>>30))*0xbf58476d1ce4e5b9ULL;x=(x^(x>>27))*0x94d049bb133111ebULL;s[i]=x^(x>>31);}}
static inline double u01(void){return ((nxt()>>11)+1)*(1.0/9007199254740993.0);} // (0,1]
static double now(void){struct timespec t;clock_gettime(CLOCK_MONOTONIC,&t);return t.tv_sec+t.tv_nsec*1e-9;}
static long rss_kb(void){FILE*f=fopen("/proc/self/status","r");char l[256];long v=0;while(f&&fgets(l,256,f))if(!strncmp(l,"VmRSS:",6))sscanf(l+6,"%ld",&v);if(f)fclose(f);return v;}
typedef struct{u64 T;u32 id;}E;
static E*h;static size_t hn;
static void push(E e){size_t i=hn++;while(i&&h[(i-1)/2].T>e.T){h[i]=h[(i-1)/2];i=(i-1)/2;}h[i]=e;}
static E pop(void){E r=h[0],e=h[--hn];size_t i=0;for(;;){size_t c=2*i+1;if(c>=hn)break;if(c+1<hn&&h[c+1].T<h[c].T)c++;if(h[c].T>=e.T)break;h[i]=h[c];i=c;}h[i]=e;return r;}
int main(int argc,char**argv){
  u64 N=strtoull(argv[1],0,10);int S=atoi(argv[2]);double q=atof(argv[3]);
  u64 ev_ticks=strtoull(argv[4],0,10),br_ticks=strtoull(argv[5],0,10);
  u64 C=N/S; double p=1-pow(1-q,1.0/S); u64 thr=(u64)(p*18446744073709551616.0);
  double inv=1.0/log(1-q);
  seed(12345);
  u32*pc=malloc(N*4);unsigned char*pt=malloc(N);for(u64 i=0;i<N;i++){pc[i]=i/S;pt[i]=1;}
  long base=rss_kb();
  // event-driven
  h=malloc(C*sizeof(E));hn=0;
  for(u64 c=0;c<C;c++){E e={(u64)floor(log(u01())*inv),(u32)c};push(e);}
  double t0=now();u64 events=0,touched=0;
  for(u64 t=0;t<ev_ticks;){ if(!hn)break; if(h[0].T>=ev_ticks)break;
    E e=pop();events++;touched+=S; E n={e.T+1+(u64)floor(log(u01())*inv),e.id};push(n);t=e.T;}
  double te=now()-t0;
  // brute component
  t0=now();u64 ev2=0;
  u64 qthr=(u64)(q*18446744073709551616.0);
  for(u64 t=0;t<br_ticks;t++)for(u64 c=0;c<C;c++)if(nxt()<qthr)ev2++;
  double tc=now()-t0;
  // brute particle
  t0=now();u64 ev3=0;
  for(u64 t=0;t<br_ticks;t++)for(u64 c=0;c<C;c++){int hit=0;for(int k=0;k<S;k++)if(nxt()<thr)hit=1;ev3+=hit;}
  double tp=now()-t0;
  printf("N=%llu S=%d comps=%llu q=%.1e | event: %llu ticks in %.3fs -> %.0f ticks/s, %.2f events/tick, simulated particles/tick=%.1f | "
         "brute_comp: %llu ticks in %.3fs -> %.0f ticks/s | brute_particle: %llu ticks in %.3fs -> %.0f ticks/s | check events/tick ev=%.3f bc=%.3f bp=%.3f | rss=%ldMB\n",
    (unsigned long long)N,S,(unsigned long long)C,q,(unsigned long long)ev_ticks,te,ev_ticks/te,(double)events/ev_ticks,(double)touched/ev_ticks,
    (unsigned long long)br_ticks,tc,br_ticks/tc,(unsigned long long)br_ticks,tp,br_ticks/tp,
    (double)events/ev_ticks,(double)ev2/br_ticks,(double)ev3/br_ticks,rss_kb()/1024);
  return 0;}

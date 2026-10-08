/* ea:   0x00053138 */
/* name: sub_53138 */
/* size: 100 */
int __fastcall sub_53138(int a1, void *a2)
{
  if ( a2 == (void *)-1 )
    return 0;
  sub_5319C((LPVOID)a1, a2);
  sub_5319C((LPVOID)(a1 + 4), a2);
  sub_53200((LPVOID)(a1 + 8), a2);
  sub_53264(a1 + 16, a2);
  sub_53338(a1 + 28, a2);
  sub_533DC((LPVOID)(a1 + 40), a2);
  return 1;
}


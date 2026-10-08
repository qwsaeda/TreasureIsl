/* ea:   0x0004EC40 */
/* name: sub_4EC40 */
/* size: 68 */
void *__fastcall sub_4EC40(_DWORD *a1, int a2, const void *a3, int a4)
{
  bool v6; // zf
  int v7; // r1

  a1[256] = a2;
  v6 = a2 == 0;
  a1[279] = a4;
  v7 = 1;
  if ( v6 )
    v7 = 2;
  sub_445F4((int)a1, v7)[5] = 0;
  return memcpy(a1 + 258, a3, 0x50u);
}


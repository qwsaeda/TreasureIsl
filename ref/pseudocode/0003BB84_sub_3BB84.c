/* ea:   0x0003BB84 */
/* name: sub_3BB84 */
/* size: 144 */
int __fastcall sub_3BB84(int a1, int a2, int a3, int a4)
{
  unsigned __int16 v5; // r7
  unsigned __int16 *v7; // r4
  int v8; // r5
  int v9; // r0

  v5 = a3;
  v7 = (unsigned __int16 *)(sub_3AA5C(a3, a4) + 136);
  v8 = rand() % 6 + 1;
  v9 = sub_16C9C(v7, 0);
  if ( v8 > v9 )
    v8 = v9;
  sub_16D78(v7, v8);
  return sub_3AB44(a1, 0, v5, a4, v8);
}


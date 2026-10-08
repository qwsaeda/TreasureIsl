/* ea:   0x0003BE24 */
/* name: sub_3BE24 */
/* size: 124 */
int __fastcall sub_3BE24(int a1, int a2, int a3, int a4)
{
  unsigned __int16 v5; // r7
  unsigned __int16 *v7; // r4
  unsigned int v8; // r5

  v5 = a3;
  v7 = (unsigned __int16 *)(sub_3AA5C(a3, a4) + 136);
  v8 = sub_16C9C(v7, 0);
  if ( v8 != 0 )
  {
    v8 /= 0xAu;
    if ( v8 == 0 )
      v8 = 1;
    sub_16D78(v7, v8);
  }
  return sub_3AB44(a1, 0, v5, a4, v8);
}


/* ea:   0x0003B0C0 */
/* name: sub_3B0C0 */
/* size: 120 */
int sub_3B0C0(_DWORD *a1, int a2, int a3, int a4, ...)
{
  int v5; // r4
  int v6; // r3
  unsigned __int8 v8; // [sp+4h] [bp-1Ch]

  v5 = a1[1];
  if ( a3 + a4 > (unsigned int)((v5 - *a1) >> 1) )
    return -1;
  v6 = sub_3B138(*a1 + 2 * a3, v5, a2, a2 + 2 * a4, v8);
  if ( v6 == a1[1] )
    return -1;
  else
    return (v6 - *a1) >> 1;
}


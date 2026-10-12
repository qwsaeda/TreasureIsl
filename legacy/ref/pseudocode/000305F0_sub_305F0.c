/* ea:   0x000305F0 */
/* name: sub_305F0 */
/* size: 304 */
unsigned int __fastcall sub_305F0(
        unsigned __int16 *a1,
        unsigned __int16 a2,
        unsigned __int16 *a3,
        unsigned __int16 *a4,
        int a5)
{
  unsigned __int16 v6; // r5
  unsigned int v8; // r2
  unsigned int v9; // r3
  unsigned int v10; // r2
  unsigned __int16 v11; // r2
  unsigned __int16 *v12; // r5
  unsigned int v13; // r3
  unsigned __int16 v14; // r0
  unsigned __int16 v15; // r4
  unsigned int v16; // r3
  unsigned int result; // r0
  int v18; // r0

  v6 = a2;
  if ( a2 > (unsigned int)*a1 )
    v6 = *a1;
  *a3 = v6;
  v8 = *a1;
  if ( v6 < v8 )
    *a1 = v8 - v6;
  else
    *a1 = 0;
  v9 = *a1;
  v10 = a1[2];
  if ( v10 > v9 )
  {
    v11 = v10 - v9;
    v12 = a1 + 2;
    *a4 = v11;
    v13 = a1[2];
    *v12 = v11 < v13 ? v13 - v11 : 0;
    v14 = *v12;
    v15 = *a1;
    a1[3] = *a1;
    if ( v14 > (unsigned int)v15 )
      *v12 = v15;
  }
  v16 = sub_16D1C((int)(a1 + 8));
  result = *a1;
  if ( v16 > result )
  {
    v18 = sub_16D1C((int)(a1 + 8));
    return sub_30720(a1 + 8, v18 - *a1, a5);
  }
  return result;
}


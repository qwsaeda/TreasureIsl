/* ea:   0x0005423C */
/* name: sub_5423C */
/* size: 332 */
int sub_5423C(int *a1, int a2, int a3, int a4, ...)
{
  unsigned int v7; // r6
  __int64 v8; // r0
  unsigned int v9; // r2
  va_list v10; // r0
  unsigned int v11; // r10
  int v12; // r0
  int v13; // r4
  int v14; // r8
  int i; // r5
  int j; // r4
  int v17; // r6
  int v18; // r5
  int v19; // r4
  unsigned int v21; // [sp+0h] [bp-3Ch] BYREF
  unsigned int v22; // [sp+3Ch] [bp+0h] BYREF
  va_list va; // [sp+3Ch] [bp+0h]
  int v24; // [sp+40h] [bp+4h]
  va_list va1; // [sp+44h] [bp+8h] BYREF

  va_start(va1, a4);
  va_start(va, a4);
  v22 = va_arg(va1, _DWORD);
  v24 = va_arg(va1, _DWORD);
  v7 = v22;
  v8 = 954437177LL * (a1[1] - *a1);
  v9 = (SHIDWORD(v8) >> 3) + (HIDWORD(v8) >> 31);
  v21 = v9;
  va_copy(v10, va);
  if ( v9 >= v22 )
    v10 = (va_list)&v21;
  v11 = v9 + *(_DWORD *)v10;
  v12 = sub_5445C(a1 + 2, v11);
  v13 = *a1;
  v14 = v12;
  for ( i = v12; v13 != a2; i += 36 )
  {
    if ( i != 0 )
      sub_5454C(i, v13);
    v13 += 36;
  }
  if ( v7 == 1 )
  {
    if ( i != 0 )
      sub_5454C(i, a3);
    j = i + 36;
  }
  else
  {
    for ( j = i; v7 != 0; j += 36 )
    {
      if ( j != 0 )
        sub_5454C(j, a3);
      --v7;
    }
  }
  if ( (_BYTE)v24 == 0 )
  {
    v17 = a1[1];
    v18 = j;
    if ( a2 != v17 )
    {
      v19 = a2 - j;
      do
      {
        if ( v18 != 0 )
          sub_5454C(v18, v19 + v18);
        v18 += 36;
      }
      while ( v19 + v18 != v17 );
    }
    j = v18;
  }
  sub_54420(a1);
  *a1 = v14;
  a1[1] = j;
  a1[2] = v14 + 36 * v11;
  return 9 * v11;
}


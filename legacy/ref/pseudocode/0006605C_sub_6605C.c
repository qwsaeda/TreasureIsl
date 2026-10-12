/* ea:   0x0006605C */
/* name: sub_6605C */
/* size: 128 */
int __fastcall sub_6605C(_DWORD *a1, __int16 a2)
{
  int v4; // r2
  int v5; // r3
  int *v6; // r0
  int result; // r0
  int v8; // [sp+0h] [bp-14h] BYREF
  int v9; // [sp+4h] [bp-10h] BYREF

  v4 = a1[1];
  if ( v4 + 2 == a1[2] )
  {
    v5 = (v4 - *a1) >> 1;
    v9 = v5;
    v8 = 1;
    v6 = &v8;
    if ( v5 != 0 )
      v6 = &v9;
    sub_66118(a1, *v6 + v5);
  }
  *(_WORD *)(a1[1] + 2) = 0;
  *(_WORD *)a1[1] = a2;
  result = a1[1] + 2;
  a1[1] = result;
  return result;
}


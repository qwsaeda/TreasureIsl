/* ea:   0x00014DC4 */
/* name: sub_14DC4 */
/* size: 372 */
int sub_14DC4(int a1, int a2, ...)
{
  int v4; // r7
  int v5; // r5
  _DWORD *v6; // r5
  int v7; // t1
  int v8; // t1
  int result; // r0
  int v10; // r0
  int v11; // r0
  _DWORD *v12; // r0
  int varg_r2; // [sp+28h] [bp-8h] BYREF
  va_list varg_r2a; // [sp+28h] [bp-8h]
  int varg_r3; // [sp+2Ch] [bp-4h]
  va_list va1; // [sp+30h] [bp+0h] BYREF

  va_start(va1, a2);
  va_start(varg_r2a, a2);
  varg_r2 = va_arg(va1, _DWORD);
  varg_r3 = va_arg(va1, _DWORD);
  v4 = varg_r3;
  sub_16B48(a2 + 164, 11);
  v5 = *(_DWORD *)(*(_DWORD *)sub_1623C(varg_r3 + 508, (int *)varg_r2a) + 32);
  v7 = *(_DWORD *)(v5 - 4);
  v6 = (_DWORD *)(v5 - 4);
  if ( v7 != 0 )
  {
LABEL_4:
    if ( (_UNKNOWN *)(a1 + 36) != &dword_81184 )
      sub_17890((int)&dword_81184);
    result = sub_3AA9C(&dword_81184, aB, *(&off_7EDD8 + *v6 - 1));
    *v6 = 0;
  }
  else
  {
    while ( v6 != *(_DWORD **)(*(_DWORD *)sub_1623C(v4 + 508, (int *)varg_r2a) + 28) )
    {
      v8 = *--v6;
      if ( v8 != 0 )
        goto LABEL_4;
    }
    if ( *(_DWORD *)(a2 + 8) == 1 )
    {
      v10 = sub_12F04(aO);
      sub_1318C((void **)&dword_81184, aO, (int)&aO[2 * v10]);
      return sub_3AA9C(&dword_81184, aC, *(_DWORD *)(a1 + 8));
    }
    else
    {
      v11 = sub_12F04(aU_0);
      sub_1318C((void **)&dword_81184, aU_0, (int)&aU_0[2 * v11]);
      sub_3AA9C(&dword_81184, aC, *(_DWORD *)(a1 + 8));
      v12 = (_DWORD *)sub_186F0(v4);
      return sub_3AA9C(&dword_81184, aU, *v12);
    }
  }
  return result;
}


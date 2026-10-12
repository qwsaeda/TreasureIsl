/* ea:   0x00014AD8 */
/* name: sub_14AD8 */
/* size: 288 */
int sub_14AD8(int a1, int a2, ...)
{
  int v4; // r6
  _DWORD *v5; // r0
  int result; // r0
  int v7; // r0
  int v8; // r0
  _DWORD *v9; // r0
  int varg_r2; // [sp+24h] [bp-8h] BYREF
  va_list varg_r2a; // [sp+24h] [bp-8h]
  int varg_r3; // [sp+28h] [bp-4h]
  va_list va1; // [sp+2Ch] [bp+0h] BYREF

  va_start(va1, a2);
  va_start(varg_r2a, a2);
  varg_r2 = va_arg(va1, _DWORD);
  varg_r3 = va_arg(va1, _DWORD);
  v4 = varg_r3;
  sub_16B48(a2 + 164, 9);
  v5 = (_DWORD *)sub_1623C(varg_r3 + 508, (int *)varg_r2a);
  if ( (unsigned __int8)sub_1822C(*v5, v4 + 32, v4 + 64) != 0 )
  {
    result = a1 + 36;
    if ( (_UNKNOWN *)(a1 + 36) != &dword_81184 )
      return sub_17890((int)&dword_81184);
  }
  else if ( *(_DWORD *)(a2 + 8) == 1 )
  {
    v7 = sub_12F04(aO);
    sub_1318C((void **)&dword_81184, aO, (int)&aO[2 * v7]);
    return sub_3AA9C(&dword_81184, aC, *(_DWORD *)(a1 + 8));
  }
  else
  {
    v8 = sub_12F04(aU_0);
    sub_1318C((void **)&dword_81184, aU_0, (int)&aU_0[2 * v8]);
    sub_3AA9C(&dword_81184, aC, *(_DWORD *)(a1 + 8));
    v9 = (_DWORD *)sub_186F0(v4);
    return sub_3AA9C(&dword_81184, aU, *v9);
  }
  return result;
}


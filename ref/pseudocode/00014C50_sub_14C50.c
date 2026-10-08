/* ea:   0x00014C50 */
/* name: sub_14C50 */
/* size: 284 */
int sub_14C50(int a1, int a2, ...)
{
  int v4; // r8
  int v5; // r0
  unsigned __int16 v6; // r5
  unsigned int v7; // r4
  int v8; // r0
  _DWORD *v9; // r0
  int varg_r2; // [sp+28h] [bp-8h] BYREF
  va_list varg_r2a; // [sp+28h] [bp-8h]
  int varg_r3; // [sp+2Ch] [bp-4h]
  va_list va1; // [sp+30h] [bp+0h] BYREF

  va_start(va1, a2);
  va_start(varg_r2a, a2);
  varg_r2 = va_arg(va1, _DWORD);
  varg_r3 = va_arg(va1, _DWORD);
  v4 = varg_r3;
  sub_16B48(a2 + 164, 10);
  v5 = sub_1623C(varg_r3 + 508, (int *)varg_r2a);
  v6 = *(unsigned __int8 *)(*(_DWORD *)v5 + 50);
  if ( *(_BYTE *)(*(_DWORD *)v5 + 50) != 0 )
  {
    v7 = *(unsigned __int8 *)(*(_DWORD *)sub_1623C(v4 + 508, (int *)varg_r2a) + 48);
    v8 = sub_3AA5C(v6, v4);
    if ( v7 < 0xA )
    {
      --*(_BYTE *)(*(_DWORD *)(v8 + 188) + v7);
      --*(_DWORD *)(v8 + 200);
    }
  }
  sub_513CC(a2, (unsigned __int16)varg_r2, 5);
  if ( (_UNKNOWN *)(a1 + 36) != &dword_81184 )
    sub_17890((int)&dword_81184);
  if ( *(_DWORD *)(a2 + 8) == 1 )
    return sub_3AA9C(&dword_81184, aU, &unk_7D724);
  v9 = (_DWORD *)sub_186F0(v4);
  return sub_3AA9C(&dword_81184, aU, *v9);
}


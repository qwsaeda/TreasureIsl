/* ea:   0x00014280 */
/* name: sub_14280 */
/* size: 232 */
int __fastcall sub_14280(int a1, int a2, int a3, int a4)
{
  int v8; // r0
  void *v9; // r2
  _DWORD *v10; // r0

  sub_16B48(a2 + 164, 4);
  v8 = sub_3AA5C(a3, a4);
  *(_DWORD *)(v8 + 176) = 8;
  *(_WORD *)(v8 + 180) = 1;
  if ( (_UNKNOWN *)(a1 + 36) != &dword_81184 )
    sub_17890((int)&dword_81184);
  if ( *(_DWORD *)(a2 + 8) == 1 )
    v9 = &unk_7D724;
  else
    v9 = *(void **)sub_186F0(a4);
  sub_3AA9C(&dword_81184, aU, v9);
  if ( (unsigned __int16)a3 == 1 )
    return sub_3AA9C(&dword_81184, aT, &unk_7D724);
  v10 = (_DWORD *)sub_186F0(a4);
  return sub_3AA9C(&dword_81184, aT, *v10);
}


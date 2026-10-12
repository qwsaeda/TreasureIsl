/* ea:   0x00015934 */
/* name: sub_15934 */
/* size: 240 */
int __fastcall sub_15934(int a1, int a2, int a3, int a4)
{
  _DWORD *v8; // r3
  bool v9; // zf
  void *v10; // r2
  _DWORD *v11; // r0

  sub_16B48(a2 + 164, 20);
  v8 = (_DWORD *)(sub_3AA5C(a3, a4) + 176);
  v9 = *v8 == 7;
  if ( *v8 != 7 )
    v9 = *v8 == 8;
  if ( !v9 )
    *v8 = 0;
  if ( (_UNKNOWN *)(a1 + 36) != &dword_81184 )
    sub_17890((int)&dword_81184);
  if ( *(_DWORD *)(a2 + 8) == 1 )
    v10 = &unk_7D724;
  else
    v10 = *(void **)sub_186F0(a4);
  sub_3AA9C(&dword_81184, aU, v10);
  if ( (unsigned __int16)a3 == 1 )
    return sub_3AA9C(&dword_81184, aT, &unk_7D724);
  v11 = (_DWORD *)sub_186F0(a4);
  return sub_3AA9C(&dword_81184, aT, *v11);
}


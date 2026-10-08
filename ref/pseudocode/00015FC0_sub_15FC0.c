/* ea:   0x00015FC0 */
/* name: sub_15FC0 */
/* size: 176 */
int __fastcall sub_15FC0(int a1, int a2, unsigned __int16 a3, int a4)
{
  int v8; // r0
  _DWORD *v9; // r0

  sub_16B48(a2 + 164, 25);
  v8 = sub_5383C(a4 + 52, a3);
  sub_52080(a2, v8);
  if ( (_UNKNOWN *)(a1 + 36) != &dword_81184 )
    sub_17890((int)&dword_81184);
  if ( *(_DWORD *)(a2 + 8) == 1 )
    return sub_3AA9C(&dword_81184, aU, &unk_7D724);
  v9 = (_DWORD *)sub_186F0(a4);
  return sub_3AA9C(&dword_81184, aU, *v9);
}


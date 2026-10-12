/* ea:   0x00015EB4 */
/* name: sub_15EB4 */
/* size: 180 */
int __fastcall sub_15EB4(int a1, int a2, int a3, int a4)
{
  int v7; // r0
  _DWORD *v8; // r0

  sub_16B48(a2 + 164, 24);
  v7 = sub_3AA5C(a3, a4);
  *(_DWORD *)(v7 + 176) = 4;
  *(_WORD *)(v7 + 180) = 3;
  if ( (_UNKNOWN *)(a1 + 36) != &dword_81184 )
    sub_17890((int)&dword_81184);
  if ( (unsigned __int16)a3 == 1 )
    return sub_3AA9C(&dword_81184, aT, &unk_7D724);
  v8 = (_DWORD *)sub_186F0(a4);
  return sub_3AA9C(&dword_81184, aT, *v8);
}


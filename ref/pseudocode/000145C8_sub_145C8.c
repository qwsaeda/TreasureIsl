/* ea:   0x000145C8 */
/* name: sub_145C8 */
/* size: 240 */
int __fastcall sub_145C8(int a1, int a2, int a3, int a4)
{
  int v7; // r4
  unsigned __int16 v8; // r0
  _WORD *v9; // r4
  unsigned __int16 v10; // t1
  unsigned __int16 v11; // r3
  _DWORD *v12; // r0

  sub_16B48(a2 + 164, 6);
  v7 = sub_3AA5C(a3, a4);
  v8 = sub_16C9C(v7 + 136, 1);
  v10 = *(_WORD *)(v7 + 140);
  v9 = (_WORD *)(v7 + 140);
  v11 = v10;
  if ( v8 < (unsigned int)v10 )
    *v9 = v11 - v8;
  else
    *v9 = 0;
  if ( (_UNKNOWN *)(a1 + 36) != &dword_81184 )
    sub_17890((int)&dword_81184);
  if ( (unsigned __int16)a3 == 1 )
    return sub_3AA9C(&dword_81184, aT, &unk_7D724);
  v12 = (_DWORD *)sub_186F0(a4);
  return sub_3AA9C(&dword_81184, aT, *v12);
}


/* ea:   0x00013CE4 */
/* name: sub_13CE4 */
/* size: 288 */
int __fastcall sub_13CE4(int a1, int a2, int a3, int a4)
{
  int v7; // r5
  int v8; // r0
  unsigned int v9; // r2
  unsigned int v10; // r1
  _DWORD *v11; // r0

  if ( a2 == 0 )
    __debugbreak();
  sub_16B48(a2 + 164, 1);
  v7 = rand() % 9999 + 1;
  v8 = sub_3AA5C(a3, a4);
  v9 = *(_DWORD *)(v8 + 148);
  v10 = v7 + *(_DWORD *)(v8 + 144);
  *(_DWORD *)(v8 + 144) = v10;
  if ( v10 > v9 )
    *(_DWORD *)(v8 + 144) = v9;
  if ( (_UNKNOWN *)(a1 + 36) != &dword_81184 )
    sub_17890((int)&dword_81184);
  if ( (unsigned __int16)a3 == 1 )
  {
    sub_3AA9C(&dword_81184, aU, &unk_7D724);
  }
  else
  {
    v11 = (_DWORD *)sub_186F0(a4);
    sub_3AA9C(&dword_81184, aU, *v11);
  }
  return sub_3AB0C((int)&dword_81184, (int)aN, v7);
}


/* ea:   0x00013E5C */
/* name: sub_13E5C */
/* size: 488 */
int __fastcall sub_13E5C(int a1, int a2, int a3, int a4)
{
  int v8; // r4
  unsigned int v9; // r3
  int v10; // r7
  void *v11; // r2
  _DWORD *v12; // r0
  int v14; // r0
  int v15; // r0
  _DWORD *v16; // r0

  sub_16B48(a2 + 164, 2);
  v8 = sub_3AA5C(a3, a4) + 136;
  v9 = sub_16C9C(v8, 2);
  if ( v9 != 0 )
  {
    v10 = 2 * (v9 / 0xA);
    if ( v10 == 0 )
      v10 = 1;
    sub_16570(v8 + 8, v10);
    sub_1654C(a2 + 144, v10);
    if ( (_UNKNOWN *)(a1 + 36) != &dword_81184 )
      sub_17890((int)&dword_81184);
    if ( *(_DWORD *)(a2 + 8) == 1 )
      v11 = &unk_7D724;
    else
      v11 = *(void **)sub_186F0(a4);
    sub_3AA9C(&dword_81184, aU, v11);
    if ( (unsigned __int16)a3 == 1 )
    {
      sub_3AA9C(&dword_81184, aT, &unk_7D724);
    }
    else
    {
      v12 = (_DWORD *)sub_186F0(a4);
      sub_3AA9C(&dword_81184, aT, *v12);
    }
    return sub_3AB0C((int)&dword_81184, (int)aN, v10);
  }
  else if ( *(_DWORD *)(a2 + 8) == 1 )
  {
    v14 = sub_12F04(aO);
    sub_1318C((void **)&dword_81184, aO, (int)&aO[2 * v14]);
    return sub_3AA9C(&dword_81184, aC, *(_DWORD *)(a1 + 8));
  }
  else
  {
    v15 = sub_12F04(aU_0);
    sub_1318C((void **)&dword_81184, aU_0, (int)&aU_0[2 * v15]);
    sub_3AA9C(&dword_81184, aC, *(_DWORD *)(a1 + 8));
    v16 = (_DWORD *)sub_186F0(a4);
    return sub_3AA9C(&dword_81184, aU, *v16);
  }
}


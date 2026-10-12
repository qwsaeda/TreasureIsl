/* ea:   0x000143C0 */
/* name: sub_143C0 */
/* size: 432 */
int __fastcall sub_143C0(int a1, int a2, int a3, int a4)
{
  int v8; // r4
  unsigned int v9; // r3
  int v10; // r6
  _DWORD *v11; // r0
  int v13; // r0
  int v14; // r0
  _DWORD *v15; // r0

  sub_16B48(a2 + 164, 5);
  v8 = sub_3AA5C(a3, a4) + 136;
  v9 = sub_16C9C(v8, 0);
  if ( v9 != 0 )
  {
    v10 = 2 * (v9 / 0xA);
    if ( v10 == 0 )
      v10 = 1;
    sub_16D78(v8, (unsigned __int16)v10);
    if ( (_UNKNOWN *)(a1 + 36) != &dword_81184 )
      sub_17890((int)&dword_81184);
    if ( (unsigned __int16)a3 == 1 )
    {
      sub_3AA9C(&dword_81184, aT, &unk_7D724);
    }
    else
    {
      v11 = (_DWORD *)sub_186F0(a4);
      sub_3AA9C(&dword_81184, aT, *v11);
    }
    return sub_3AB0C((int)&dword_81184, (int)aR, v10);
  }
  else if ( *(_DWORD *)(a2 + 8) == 1 )
  {
    v13 = sub_12F04(aO);
    sub_1318C((void **)&dword_81184, aO, (int)&aO[2 * v13]);
    return sub_3AA9C(&dword_81184, aC, *(_DWORD *)(a1 + 8));
  }
  else
  {
    v14 = sub_12F04(aU_0);
    sub_1318C((void **)&dword_81184, aU_0, (int)&aU_0[2 * v14]);
    sub_3AA9C(&dword_81184, aC, *(_DWORD *)(a1 + 8));
    v15 = (_DWORD *)sub_186F0(a4);
    return sub_3AA9C(&dword_81184, aU, *v15);
  }
}


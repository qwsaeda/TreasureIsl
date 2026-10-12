/* ea:   0x00014710 */
/* name: sub_14710 */
/* size: 508 */
int __fastcall sub_14710(int a1, int a2, int a3, int a4)
{
  int v8; // r5
  unsigned int v9; // r3
  unsigned int v10; // r7
  void *v11; // r2
  _DWORD *v12; // r0
  int v14; // r0
  int v15; // r0
  _DWORD *v16; // r0

  sub_16B48(a2 + 164, 7);
  v8 = sub_3AA5C(a3, a4) + 136;
  v9 = sub_16C9C(v8, 0);
  if ( v9 != 0 )
  {
    v10 = v9 / 0xA;
    if ( v9 / 0xA == 0 )
      v10 = 1;
    sub_16D78(v8, (unsigned __int16)v10);
    sub_16500(a2 + 136, (unsigned __int16)v10);
    sub_164D4(a2 + 140, *(unsigned __int16 *)(a2 + 136));
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
    return sub_3AB0C((int)&dword_81184, (int)aR, v10);
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


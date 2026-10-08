/* ea:   0x00015538 */
/* name: sub_15538 */
/* size: 532 */
int __fastcall sub_15538(int a1, int a2, int a3, int a4)
{
  int v8; // r6
  int v9; // r0
  _DWORD *v10; // r5
  int v11; // r4
  int v12; // r0
  void *v13; // r2
  _DWORD *v14; // r0
  int v16; // r0
  int v17; // r0
  _DWORD *v18; // r0
  char v19; // [sp+0h] [bp-24h] BYREF

  v8 = a2 + 164;
  sub_16B48(a2 + 164, 17);
  v9 = sub_3AA5C(a3, a4);
  v10 = (_DWORD *)(v9 + 164);
  v11 = 0;
  if ( *(_DWORD *)(v9 + 164) != *(_DWORD *)(v9 + 168) )
  {
    v11 = *(unsigned __int8 *)(rand() % (unsigned int)(*(_DWORD *)(v9 + 168) - *(_DWORD *)(v9 + 164)) + *v10);
    sub_16B48(v10, v11);
    v19 = v11;
    sub_161DC(v8, &v19);
  }
  if ( (_BYTE)v11 != 0 )
  {
    if ( (_UNKNOWN *)(a1 + 36) != &dword_81184 )
      sub_17890((int)&dword_81184);
    v12 = sub_173C4(a4 + 652, v11);
    sub_3AA9C(&dword_81184, aC, *(_DWORD *)(v12 + 8));
    if ( *(_DWORD *)(a2 + 8) == 1 )
      v13 = &unk_7D724;
    else
      v13 = *(void **)sub_186F0(a4);
    sub_3AA9C(&dword_81184, aU, v13);
    if ( (unsigned __int16)a3 == 1 )
    {
      return sub_3AA9C(&dword_81184, aT, &unk_7D724);
    }
    else
    {
      v14 = (_DWORD *)sub_186F0(a4);
      return sub_3AA9C(&dword_81184, aT, *v14);
    }
  }
  else if ( *(_DWORD *)(a2 + 8) == 1 )
  {
    v16 = sub_12F04(aO);
    sub_1318C((void **)&dword_81184, aO, (int)&aO[2 * v16]);
    return sub_3AA9C(&dword_81184, aC, *(_DWORD *)(a1 + 8));
  }
  else
  {
    v17 = sub_12F04(aU_0);
    sub_1318C((void **)&dword_81184, aU_0, (int)&aU_0[2 * v17]);
    sub_3AA9C(&dword_81184, aC, *(_DWORD *)(a1 + 8));
    v18 = (_DWORD *)sub_186F0(a4);
    return sub_3AA9C(&dword_81184, aU, *v18);
  }
}


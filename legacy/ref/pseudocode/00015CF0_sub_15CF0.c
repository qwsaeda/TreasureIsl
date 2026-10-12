/* ea:   0x00015CF0 */
/* name: sub_15CF0 */
/* size: 364 */
int __fastcall sub_15CF0(int a1, int a2, int a3, int a4)
{
  _DWORD *v8; // r0
  int v10; // r0
  int v11; // r0
  _DWORD *v12; // r0

  sub_16B48(a2 + 164, 23);
  if ( *(_DWORD *)(sub_3AA5C(a3, a4) + 176) == 6 )
  {
    if ( *(_DWORD *)(a2 + 8) == 1 )
    {
      v10 = sub_12F04(aO);
      sub_1318C((void **)&dword_81184, aO, (int)&aO[2 * v10]);
      return sub_3AA9C(&dword_81184, aC, *(_DWORD *)(a1 + 8));
    }
    else
    {
      v11 = sub_12F04(aU_0);
      sub_1318C((void **)&dword_81184, aU_0, (int)&aU_0[2 * v11]);
      sub_3AA9C(&dword_81184, aC, *(_DWORD *)(a1 + 8));
      v12 = (_DWORD *)sub_186F0(a4);
      return sub_3AA9C(&dword_81184, aU, *v12);
    }
  }
  else
  {
    sub_2F160(a4, *(_DWORD *)(a2 + 8), (unsigned __int16)a3);
    if ( (_UNKNOWN *)(a1 + 36) != &dword_81184 )
      sub_17890((int)&dword_81184);
    if ( (unsigned __int16)a3 == 1 )
    {
      return sub_3AA9C(&dword_81184, aT, &unk_7D724);
    }
    else
    {
      v8 = (_DWORD *)sub_186F0(a4);
      return sub_3AA9C(&dword_81184, aT, *v8);
    }
  }
}


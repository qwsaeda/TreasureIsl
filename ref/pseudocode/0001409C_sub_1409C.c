/* ea:   0x0001409C */
/* name: sub_1409C */
/* size: 396 */
int __fastcall sub_1409C(int a1, int a2, int a3, int a4)
{
  int v8; // r4
  int v9; // r8
  int v10; // r9
  int v11; // r0
  int v12; // r0
  unsigned int *v13; // r4
  int v14; // t1
  unsigned int v15; // r2
  unsigned int v16; // r1
  unsigned int v17; // r0
  _DWORD *v18; // r4
  unsigned int v19; // t1
  void *v20; // r2
  void *v21; // r2
  int v22; // r0

  sub_16B48(a2 + 164, 3);
  v8 = sub_3AA5C(a3, a4);
  v9 = v8 + 136;
  v10 = rand() % 6 + 1;
  v11 = v8 + 136;
  if ( v10 % 2 != 0 )
  {
    v16 = sub_16C9C(v11, 2) + 1;
    v19 = *(_DWORD *)(v8 + 144);
    v18 = (_DWORD *)(v8 + 144);
    v17 = v19;
    if ( v16 >> 1 < v19 )
      *v18 = v17 - (v16 >> 1);
    else
      *v18 = 0;
  }
  else
  {
    v12 = sub_16C9C(v11, 2);
    v14 = *(_DWORD *)(v8 + 144);
    v13 = (unsigned int *)(v8 + 144);
    v15 = v13[1];
    *v13 = v14 + v12;
    if ( v14 + v12 > v15 )
      *v13 = v15;
  }
  if ( (_UNKNOWN *)(a1 + 36) != &dword_81184 )
    sub_17890((int)&dword_81184);
  if ( *(_DWORD *)(a2 + 8) == 1 )
    v20 = &unk_7D724;
  else
    v20 = *(void **)sub_186F0(a4);
  sub_3AA9C(&dword_81184, aU, v20);
  if ( (unsigned __int16)a3 == 1 )
    v21 = &unk_7D724;
  else
    v21 = *(void **)sub_186F0(a4);
  sub_3AA9C(&dword_81184, aT, v21);
  sub_3AB0C((int)&dword_81184, (int)aR, v10);
  v22 = sub_16C9C(v9, 2);
  return sub_3AB0C((int)&dword_81184, (int)aM, v22);
}


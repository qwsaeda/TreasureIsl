/* ea:   0x0004CC70 */
/* name: sub_4CC70 */
/* size: 824 */
void __fastcall sub_4CC70(int a1, int a2)
{
  unsigned __int16 *v4; // r9
  int v5; // r8
  int v6; // r6
  __int16 v7; // r7
  int v8; // r10
  int v9; // r3
  unsigned int v10; // r2
  __int16 v11; // r0
  int v12; // r3
  unsigned int v13; // r2
  unsigned int v14; // r2
  int v15; // r4
  unsigned __int16 v16; // r5
  int v17; // r2
  __int16 v18; // r3
  int v19; // r3
  unsigned int v20; // r2
  unsigned __int16 v21; // r4
  unsigned __int16 v22; // r5
  unsigned __int16 v23; // r4
  int v24; // r0
  unsigned int v25; // r1
  int v26; // r2
  unsigned int v27; // r4
  unsigned __int16 v28[4]; // [sp+4h] [bp-38h] BYREF
  _BYTE v29[48]; // [sp+Ch] [bp-30h] BYREF

  v4 = (unsigned __int16 *)(a1 + 136);
  v5 = *(unsigned __int16 *)(a2 + 16) - sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  v6 = *(unsigned __int16 *)(a2 + 18) - sub_16C9C(v4, 1);
  v7 = 0;
  v8 = *(_DWORD *)(a2 + 12) - sub_16C9C(v4, 2);
  v28[0] = 0;
  sub_32134((int)v29);
  if ( v6 <= 0 )
  {
    if ( v5 <= 0 )
    {
      v22 = abs32(v5);
      sub_30A7C(*(_DWORD *)(a1 + 16), v22);
      v23 = abs32(v6);
      sub_30AD0(*(_DWORD *)(a1 + 16), v23);
      sub_357C0((_WORD *)(a1 + 140), v23);
      sub_305F0(v4, v22, v28, v28, (int)v29);
    }
    else
    {
      v19 = *(_DWORD *)(a1 + 16);
      v20 = *(unsigned __int16 *)(v19 + 12);
      if ( (unsigned __int16)v5 <= v20 )
        v7 = v20 - v5;
      *(_WORD *)(v19 + 12) = v7;
      v21 = abs32(v6);
      sub_30AD0(*(_DWORD *)(a1 + 16), v21);
      sub_3053C(v4, v5);
      sub_357C0((_WORD *)(a1 + 140), v21);
    }
  }
  else
  {
    v9 = *(_DWORD *)(a1 + 16);
    if ( v5 <= 0 )
    {
      v14 = *(unsigned __int16 *)(v9 + 14);
      if ( (unsigned __int16)v6 <= v14 )
        v7 = v14 - v6;
      *(_WORD *)(v9 + 14) = v7;
      v15 = *(_DWORD *)(a1 + 16);
      v16 = abs32(v5);
      v17 = *(unsigned __int16 *)(v15 + 12);
      v18 = 999;
      if ( v17 + (unsigned int)v16 <= 0x3E7 )
        v18 = v17 + v16;
      *(_WORD *)(v15 + 12) = v18;
      sub_305F0(v4, v16, v28, v28, (int)v29);
      sub_164D4(v4 + 2, *v4);
      sub_16500(v4 + 2, v6);
    }
    else
    {
      v10 = *(unsigned __int16 *)(v9 + 14);
      v11 = 0;
      if ( (unsigned __int16)v6 <= v10 )
        v11 = v10 - v6;
      *(_WORD *)(v9 + 14) = v11;
      v12 = *(_DWORD *)(a1 + 16);
      v13 = *(unsigned __int16 *)(v12 + 12);
      if ( (unsigned __int16)v5 <= v13 )
        v7 = v13 - v5;
      *(_WORD *)(v12 + 12) = v7;
      sub_16500(v4, v5);
      sub_164D4(v4 + 2, *v4);
      sub_164D4(v4 + 2, *v4);
      sub_16500(v4 + 2, v6);
    }
  }
  v24 = sub_51C14(a1, v29);
  sub_1654C((unsigned int *)(a1 + 144), v24);
  if ( v8 <= 0 )
  {
    v27 = abs32(v8);
    sub_16570((unsigned int *)(a1 + 144), v27);
    sub_30A44(*(_DWORD *)(a1 + 16), v27);
  }
  else
  {
    sub_4DF94(*(_DWORD *)(a1 + 16), v8);
    sub_1654C((unsigned int *)(a1 + 144), v8);
  }
  CFrameWnd::OnSysCommand((CFrameWnd *)v29, v25, v26);
}


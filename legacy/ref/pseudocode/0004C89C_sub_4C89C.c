/* ea:   0x0004C89C */
/* name: sub_4C89C */
/* size: 756 */
void __fastcall sub_4C89C(int a1, int a2)
{
  unsigned __int16 *v4; // r8
  int v5; // r7
  int v6; // r3
  unsigned __int16 v7; // r0
  __int16 v8; // r4
  int v9; // r6
  int v10; // r3
  unsigned int v11; // r2
  int v12; // r0
  unsigned int v13; // r1
  int v14; // r2
  unsigned int v15; // r2
  int v16; // r4
  unsigned __int16 v17; // r5
  int v18; // r2
  __int16 v19; // r3
  int v20; // r0
  unsigned int v21; // r1
  int v22; // r2
  unsigned __int16 v23; // r4
  int v24; // r0
  unsigned int v25; // r1
  int v26; // r2
  unsigned __int16 v27; // r5
  unsigned __int16 v28; // r4
  int v29; // r0
  unsigned int v30; // r1
  int v31; // r2
  unsigned __int16 v32[4]; // [sp+4h] [bp-30h] BYREF
  _BYTE v33[40]; // [sp+Ch] [bp-28h] BYREF

  v4 = (unsigned __int16 *)(a1 + 136);
  v5 = *(unsigned __int16 *)(a2 + 16) - sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  v6 = sub_16C9C(v4, 1);
  v7 = *(_WORD *)(a2 + 18);
  v8 = 0;
  v32[0] = 0;
  v9 = v7 - v6;
  sub_32134((int)v33);
  if ( v9 <= 0 )
  {
    if ( v5 <= 0 )
    {
      v27 = abs32(v5);
      sub_311C0(*(_DWORD *)(a1 + 20), v27);
      v28 = abs32(v9);
      sub_31258(*(_DWORD *)(a1 + 20), v28);
      sub_357C0((_WORD *)(a1 + 140), v28);
      sub_305F0(v4, v27, v32, v32, (int)v33);
      v29 = sub_51C14(a1, v33);
      sub_1654C((unsigned int *)(a1 + 144), v29);
      CFrameWnd::OnSysCommand((CFrameWnd *)v33, v30, v31);
    }
    else
    {
      sub_31200(*(_DWORD *)(a1 + 20), v5);
      v23 = abs32(v9);
      sub_31258(*(_DWORD *)(a1 + 20), v23);
      sub_3053C(v4, v5);
      sub_357C0((_WORD *)(a1 + 140), v23);
      v24 = sub_51C14(a1, v33);
      sub_1654C((unsigned int *)(a1 + 144), v24);
      CFrameWnd::OnSysCommand((CFrameWnd *)v33, v25, v26);
    }
  }
  else
  {
    v10 = *(_DWORD *)(a1 + 20);
    if ( v5 <= 0 )
    {
      v15 = *(unsigned __int16 *)(v10 + 24);
      if ( (unsigned __int16)v9 <= v15 )
        v8 = v15 - v9;
      *(_WORD *)(v10 + 24) = v8;
      v16 = *(_DWORD *)(a1 + 20);
      v17 = abs32(v5);
      v18 = *(unsigned __int16 *)(v16 + 22);
      v19 = 999;
      if ( v18 + (unsigned int)v17 <= 0x3E7 )
        v19 = v18 + v17;
      *(_WORD *)(v16 + 22) = v19;
      sub_305F0(v4, v17, v32, v32, (int)v33);
      sub_164D4(v4 + 2, *v4);
      sub_16500(v4 + 2, v9);
      v20 = sub_51C14(a1, v33);
      sub_1654C((unsigned int *)(a1 + 144), v20);
      CFrameWnd::OnSysCommand((CFrameWnd *)v33, v21, v22);
    }
    else
    {
      v11 = *(unsigned __int16 *)(v10 + 24);
      if ( (unsigned __int16)v9 <= v11 )
        v8 = v11 - v9;
      *(_WORD *)(v10 + 24) = v8;
      sub_31200(*(_DWORD *)(a1 + 20), v5);
      sub_16500(v4, v5);
      sub_164D4(v4 + 2, *v4);
      sub_164D4(v4 + 2, *v4);
      sub_16500(v4 + 2, v9);
      v12 = sub_51C14(a1, v33);
      sub_1654C((unsigned int *)(a1 + 144), v12);
      CFrameWnd::OnSysCommand((CFrameWnd *)v33, v13, v14);
    }
  }
}


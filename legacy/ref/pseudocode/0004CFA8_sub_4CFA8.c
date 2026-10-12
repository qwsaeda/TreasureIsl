/* ea:   0x0004CFA8 */
/* name: sub_4CFA8 */
/* size: 248 */
void __fastcall sub_4CFA8(int a1, int a2)
{
  unsigned __int16 *v4; // r6
  unsigned int v5; // r5
  __int16 v6; // r0
  int v7; // r1
  int v8; // r1
  int v9; // r2
  int v10; // r0
  unsigned __int16 v11; // r1
  unsigned int *v12; // r4
  unsigned int v13; // r0
  int v14; // r0
  unsigned int v15; // r2
  unsigned int v16; // r1
  char v17; // [sp+4h] [bp-2Ch] BYREF
  _BYTE v18[2]; // [sp+6h] [bp-2Ah] BYREF
  unsigned __int16 v19; // [sp+8h] [bp-28h] BYREF
  unsigned __int16 v20; // [sp+Ah] [bp-26h] BYREF
  _BYTE v21[36]; // [sp+Ch] [bp-24h] BYREF

  v4 = (unsigned __int16 *)(a1 + 136);
  v5 = sub_16C9C((unsigned __int16 *)(a1 + 136), 2) - *(_DWORD *)a2;
  v6 = sub_16C9C(v4, 0);
  v7 = *(unsigned __int16 *)(a2 + 24) << 16;
  v20 = 0;
  v19 = v6 - HIWORD(v7);
  sub_30038((int)v21, v18);
  v8 = *(_DWORD *)(a2 + 12);
  v9 = *(_DWORD *)(a2 + 16);
  v10 = *(_DWORD *)(a1 + 20);
  v17 = v18[0];
  sub_4DFB4(v10 + 28, v8, v9, &v17);
  v11 = v19;
  v12 = (unsigned int *)(a1 + 144);
  v13 = *(_DWORD *)(a1 + 144);
  if ( v5 >= v13 )
    *v12 = 0;
  else
    *v12 = v13 - v5;
  sub_305F0(v4, v11, &v19, &v20, (int)v21);
  v14 = sub_51C14(a1, v21);
  v15 = *(_DWORD *)(a1 + 148);
  v16 = *v12 + v14;
  *v12 = v16;
  if ( v16 > v15 )
    *v12 = v15;
  CFrameWnd::OnSysCommand((CFrameWnd *)v21, v16, v15);
}


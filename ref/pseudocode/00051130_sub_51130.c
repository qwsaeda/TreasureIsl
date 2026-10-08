/* ea:   0x00051130 */
/* name: sub_51130 */
/* size: 668 */
void __fastcall sub_51130(int a1, unsigned __int16 a2)
{
  _DWORD *v4; // r6
  int v5; // r4
  int v6; // r2
  __int16 v7; // r0
  int v8; // r3
  unsigned int v9; // r3
  int v10; // r0
  int v11; // r5
  unsigned int v12; // kr00_4
  unsigned int v13; // r4
  int v14; // r0
  _DWORD *v15; // r4
  int v16; // r3
  int v17; // r2
  int v18; // r5
  _DWORD *v19; // r0
  int v20; // r0
  unsigned int v21; // r1
  int v22; // r2
  unsigned __int16 v23; // [sp+4h] [bp-44h] BYREF
  unsigned __int16 v24; // [sp+6h] [bp-42h] BYREF
  int v25; // [sp+8h] [bp-40h] BYREF
  _BYTE v26[16]; // [sp+Ch] [bp-3Ch] BYREF
  int v27; // [sp+1Ch] [bp-2Ch] BYREF
  int v28; // [sp+20h] [bp-28h]
  int v29; // [sp+24h] [bp-24h]
  int v30; // [sp+28h] [bp-20h]
  int v31; // [sp+2Ch] [bp-1Ch]
  int v32; // [sp+30h] [bp-18h]

  if ( *(_DWORD *)(a1 + 20) != 0 && sub_16C9C((unsigned __int16 *)(a1 + 136), 0) >= (unsigned int)a2 && a2 != 0 )
  {
    v4 = nullptr;
    v23 = 0;
    v24 = 0;
    sub_32134((int)v26);
    sub_305F0((unsigned __int16 *)(a1 + 136), a2, &v23, &v24, (int)v26);
    v5 = *(_DWORD *)(a1 + 20);
    v6 = *(unsigned __int16 *)(v5 + 22);
    v7 = 999;
    if ( v6 + (unsigned int)v23 <= 0x3E7 )
      v7 = v6 + v23;
    *(_WORD *)(v5 + 22) = v7;
    sub_31258(*(_DWORD *)(a1 + 20), v24);
    v8 = *(_DWORD *)(a1 + 8);
    if ( v8 <= 4 )
      *(_BYTE *)(*(_DWORD *)(a1 + 20) + 50) = v8;
    v9 = *(unsigned __int8 *)(*(_DWORD *)(a1 + 20) + 48);
    if ( v9 < 0xA )
    {
      ++*(_BYTE *)(v9 + *(_DWORD *)(a1 + 188));
      ++*(_DWORD *)(a1 + 200);
    }
    v10 = sub_51C14(a1, v26);
    sub_1654C((unsigned int *)(a1 + 144), v10);
    if ( *(_BYTE *)(*(_DWORD *)(a1 + 20) + 49) == 0 )
    {
      v11 = *(_DWORD *)(a1 + 24) + 144;
      v12 = *(_DWORD *)(*(_DWORD *)(a1 + 24) + 156);
      v13 = *(_DWORD *)(*(_DWORD *)(a1 + 24) + 152) + 1;
      *(_DWORD *)(*(_DWORD *)(a1 + 24) + 152) = v13;
      if ( v13 / 7 >= v12 / 0xA )
        *(_DWORD *)(v11 + 28) = 1;
      v14 = *(_DWORD *)(a1 + 20);
      v27 = 0;
      v15 = (_DWORD *)(v14 + 8);
      v28 = 0;
      v29 = 0;
      v30 = 0;
      v31 = 0;
      v32 = 0;
      sub_5350C();
      v16 = *(_DWORD *)(a1 + 20);
      v17 = *(_DWORD *)(*(_DWORD *)(a1 + 24) + 32) + 4 * *(unsigned __int8 *)(v16 + 49);
      if ( *(_DWORD *)(v16 + 44) == 1 )
        v18 = *(_DWORD *)(v17 + 8);
      else
        v18 = *(_DWORD *)(v17 + 24);
      if ( v18 == 0 )
        __debugbreak();
      v28 = *(_DWORD *)(v16 + 12) - *(_DWORD *)(v18 + 4) + 64;
      v29 = *(_DWORD *)(v16 + 16) - *(_DWORD *)(v18 + 8) + 32;
      v19 = operator new(0x14u);
      if ( v19 != nullptr )
        v4 = sub_430AC(v19);
      *v15 = v4;
      if ( v4 != nullptr )
      {
        sub_11558(*(_DWORD *)(a1 + 24) + 64, &v25, *(_DWORD *)(*(_DWORD *)(a1 + 24) + 64), v15);
        v20 = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(*(_DWORD *)(a1 + 24) + 64) + 4) + 8);
        (*(void (__fastcall **)(int, int, int *, int))(*(_DWORD *)v20 + 8))(v20, v18, &v27, 1);
      }
      sub_3958C((_DWORD **)(*(_DWORD *)(a1 + 24) + 64));
    }
    (*(void (__fastcall **)(_DWORD, _DWORD))(**(_DWORD **)(*(_DWORD *)(a1 + 12) + 40) + 32))(
      *(_DWORD *)(*(_DWORD *)(a1 + 12) + 40),
      *(_DWORD *)(a1 + 60));
    CFrameWnd::OnSysCommand((CFrameWnd *)v26, v21, v22);
  }
}


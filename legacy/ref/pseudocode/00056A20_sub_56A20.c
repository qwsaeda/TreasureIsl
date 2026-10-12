/* ea:   0x00056A20 */
/* name: sub_56A20 */
/* size: 752 */
_DWORD **__fastcall sub_56A20(int a1, int a2, char a3)
{
  unsigned __int16 v5; // r4
  unsigned __int16 v6; // r6
  int v7; // r2
  __int16 v8; // r3
  unsigned int v9; // r3
  int v10; // r4
  int v11; // r0
  char v12; // r0
  _BYTE *v13; // r3
  char v14; // t1
  unsigned int v15; // r5
  int v16; // r3
  char v17; // t1
  int v18; // r1
  unsigned int v19; // r4
  int v20; // r0
  int v21; // r3
  int v22; // r0
  int v23; // r0
  bool v24; // zf
  int v25; // r0
  int v26; // r2
  int v27; // r1
  int v28; // r5
  _DWORD *v29; // r0
  _DWORD *v30; // r0
  int v31; // r0
  _DWORD **result; // r0
  int v33; // [sp+0h] [bp-3Ch] BYREF
  int v34; // [sp+4h] [bp-38h] BYREF
  int v35; // [sp+8h] [bp-34h] BYREF
  int v36; // [sp+Ch] [bp-30h]
  int v37; // [sp+10h] [bp-2Ch]
  int v38; // [sp+14h] [bp-28h]
  int v39; // [sp+18h] [bp-24h]
  int v40; // [sp+1Ch] [bp-20h]

  if ( a2 == 0 || a3 == 0 )
    goto LABEL_31;
  v5 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  v6 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
  sub_357C0((_WORD *)(a1 + 140), v6);
  sub_16D78((unsigned __int16 *)(a1 + 136), v5);
  v7 = *(unsigned __int16 *)(a2 + 22);
  v8 = 999;
  if ( v7 + (unsigned int)v5 <= 0x3E7 )
    v8 = v7 + v5;
  *(_WORD *)(a2 + 22) = v8;
  sub_31258(a2, v6);
  if ( *(_DWORD *)(a1 + 216) == 1 )
  {
    v9 = *(unsigned __int8 *)(a2 + 48);
    v10 = *(_DWORD *)(a1 + 24) + 288;
    if ( v9 < 0xA )
    {
      v11 = *(_DWORD *)(*(_DWORD *)(a1 + 24) + 476);
      v14 = *(_BYTE *)(v9 + v11);
      v13 = (_BYTE *)(v9 + v11);
      v12 = v14;
LABEL_10:
      *v13 = v12 + 1;
      ++*(_DWORD *)(v10 + 200);
    }
  }
  else
  {
    v15 = *(unsigned __int8 *)(a2 + 48);
    v10 = *(_DWORD *)(*sub_1E48C(*(_DWORD *)(a1 + 24) + 496, &v34, a1 + 216) + 20);
    if ( v15 < 0xA )
    {
      v16 = *(_DWORD *)(v10 + 188);
      v17 = *(_BYTE *)(v16 + v15);
      v13 = (_BYTE *)(v16 + v15);
      v12 = v17;
      goto LABEL_10;
    }
  }
  if ( *(_BYTE *)(a2 + 50) == 1 )
  {
    v18 = *(_DWORD *)(a1 + 24);
    if ( *(unsigned __int8 *)(a2 + 48) < 0xAu )
    {
      --*(_BYTE *)(*(unsigned __int8 *)(a2 + 48) + *(_DWORD *)(v18 + 476));
      --*(_DWORD *)(v18 + 488);
    }
  }
  else
  {
    v19 = *(unsigned __int8 *)(a2 + 48);
    v20 = *(_DWORD *)(a1 + 24);
    v33 = *(unsigned __int8 *)(a2 + 50);
    v21 = *(_DWORD *)(*sub_1E48C(v20 + 496, &v34, (int)&v33) + 20);
    if ( v19 < 0xA )
    {
      --*(_BYTE *)(v19 + *(_DWORD *)(v21 + 188));
      --*(_DWORD *)(v21 + 200);
    }
  }
  v22 = *(_DWORD *)(a1 + 216);
  if ( v22 <= 4 )
    *(_BYTE *)(a2 + 50) = v22;
  if ( *(_BYTE *)(a2 + 49) == 0 )
  {
    sub_5350C(a2);
    v23 = *(_DWORD *)(a2 + 44);
    v36 = 0;
    v24 = v23 == 1;
    v25 = *(_DWORD *)(a1 + 24);
    v35 = 0;
    v37 = 0;
    v38 = 0;
    v39 = 0;
    v40 = 0;
    v26 = *(_DWORD *)(v25 + 32) + 4 * *(unsigned __int8 *)(a2 + 49);
    v27 = *(_DWORD *)(a1 + 20);
    if ( v24 )
      v28 = *(_DWORD *)(v26 + 8);
    else
      v28 = *(_DWORD *)(v26 + 24);
    v36 = *(_DWORD *)(v27 + 12);
    if ( v28 == 0 )
      __debugbreak();
    v37 = *(_DWORD *)(v27 + 16) - *(_DWORD *)(v28 + 8) + 32;
    v29 = operator new(0x14u);
    if ( v29 != nullptr )
      v30 = sub_430AC(v29);
    else
      v30 = nullptr;
    *(_DWORD *)(a2 + 8) = v30;
    if ( v30 != nullptr )
    {
      sub_11558(*(_DWORD *)(a1 + 24) + 64, &v34, *(_DWORD *)(*(_DWORD *)(a1 + 24) + 64), (_DWORD *)(a2 + 8));
      v31 = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(*(_DWORD *)(a1 + 24) + 64) + 4) + 8);
      (*(void (__fastcall **)(int, int, int *, int))(*(_DWORD *)v31 + 8))(v31, v28, &v35, 1);
    }
    sub_3958C((_DWORD **)(*(_DWORD *)(a1 + 24) + 64));
  }
  (*(void (__fastcall **)(_DWORD, _DWORD))(**(_DWORD **)(*(_DWORD *)(a1 + 12) + 40) + 32))(
    *(_DWORD *)(*(_DWORD *)(a1 + 12) + 40),
    *(_DWORD *)(a1 + 60));
LABEL_31:
  result = sub_52CE8(a1 + 184);
  *(_BYTE *)(a1 + 4) = 0;
  return result;
}


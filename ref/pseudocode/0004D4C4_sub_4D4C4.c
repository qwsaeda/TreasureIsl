/* ea:   0x0004D4C4 */
/* name: sub_4D4C4 */
/* size: 556 */
int __fastcall sub_4D4C4(_DWORD *a1, int a2, int a3, int a4)
{
  int result; // r0
  void *v9; // r0
  int v10; // r0
  int v11; // r6
  __int16 v12; // r5
  int v13; // r4
  unsigned int v14; // r2
  unsigned int v15; // r1
  __int16 v16; // r0
  int v17; // r4
  unsigned int v18; // r3
  unsigned int v19; // r1
  __int16 v20; // r0
  int v21; // r3
  int v22; // r1
  int v23; // r0
  int v24; // [sp+8h] [bp-24h] BYREF
  int v25; // [sp+Ch] [bp-20h] BYREF
  int v26; // [sp+10h] [bp-1Ch] BYREF

  result = *(unsigned __int16 *)(a1[4] + 16);
  if ( result == 0 )
    return result;
  v24 = *(_DWORD *)(a1[6] + 724) + 1;
  v9 = operator new(0xDCu);
  if ( v9 == nullptr )
  {
    v11 = 0;
LABEL_13:
    (*(void (__fastcall **)(int))(*(_DWORD *)v11 + 16))(v11);
    return (**(int (__fastcall ***)(int, int))v11)(v11, 1);
  }
  v10 = sub_55728(v9, a3, a4, a1[2], *(unsigned __int16 *)(a2 + 24), *(unsigned __int16 *)(a2 + 26));
  v11 = v10;
  if ( v10 == 0
    || (*(unsigned __int8 (__fastcall **)(int, _DWORD, int, int))(*(_DWORD *)v10 + 48))(v10, a1[6], 3, v24) == 0 )
  {
    goto LABEL_13;
  }
  v12 = 0;
  ++*(_DWORD *)(a1[6] + 724);
  v13 = a1[4];
  v14 = *(unsigned __int16 *)(v13 + 12);
  v15 = *(unsigned __int16 *)(a2 + 24);
  v16 = 0;
  if ( v15 <= v14 )
    v16 = v14 - v15;
  *(_WORD *)(v13 + 12) = v16;
  v17 = a1[4];
  v18 = *(unsigned __int16 *)(v17 + 14);
  v19 = *(unsigned __int16 *)(a2 + 26);
  v20 = 0;
  if ( v19 <= v18 )
    v20 = v18 - v19;
  *(_WORD *)(v17 + 14) = v20;
  v21 = a1[4];
  if ( *(_WORD *)(v21 + 16) != 0 )
    v12 = *(_WORD *)(v21 + 16) - 1;
  *(_WORD *)(v21 + 16) = v12;
  *(_DWORD *)sub_1DD10((_DWORD *)(a1[6] + 496), &v24) = v11;
  *(_DWORD *)sub_2FA58((_DWORD *)(a1[6] + 532), &v24) = 56 - *(_DWORD *)(a1[3] + 12);
  *(_DWORD *)(sub_2FA58((_DWORD *)(a1[6] + 532), &v24) + 4) = 94 - *(_DWORD *)(a1[3] + 16);
  v22 = *(_DWORD *)sub_1DD10((_DWORD *)(a1[6] + 496), &v24);
  v23 = a1[6];
  v25 = v22;
  sub_11558(v23 + 64, &v26, *(_DWORD *)(v23 + 64), &v25);
  return (int)sub_3958C((_DWORD **)(a1[6] + 64));
}


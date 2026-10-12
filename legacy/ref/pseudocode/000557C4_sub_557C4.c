/* ea:   0x000557C4 */
/* name: sub_557C4 */
/* size: 588 */
int __fastcall sub_557C4(int a1, int a2, int a3, int a4)
{
  int v7; // r0
  int v9; // r0
  int v10; // r2
  _DWORD *v11; // r7
  int v12; // r5
  int v13; // r2
  int v14; // r0
  int v15; // r6
  int v16; // r8
  int v17; // r4
  int v18; // r0
  int v19; // r1
  unsigned int v20; // r1
  int v21; // [sp+0h] [bp-30h] BYREF
  int v22; // [sp+4h] [bp-2Ch] BYREF
  int v23; // [sp+8h] [bp-28h] BYREF

  *(_DWORD *)(a1 + 24) = a2;
  if ( a2 == 0 )
    return 0;
  v7 = sub_53890((int *)(a2 + 52), *(_DWORD *)(a1 + 216));
  *(_DWORD *)(a1 + 12) = v7;
  if ( v7 == 0 )
    return 0;
  v9 = *(_DWORD *)(a1 + 24);
  *(_DWORD *)(a1 + 20) = 0;
  v10 = *(_DWORD *)(*sub_52D98(v9 + 520, &v22, (int *)(a1 + 216)) + 20);
  *(_DWORD *)(a1 + 16) = v10;
  if ( v10 == 0 )
    return 0;
  if ( a4 <= 4 )
    return 0;
  v11 = *(_DWORD **)(a1 + 24);
  v12 = *(_DWORD *)(a1 + 216);
  *(_DWORD *)(a1 + 44) = *(_DWORD *)(v11[8] + 108);
  *(_DWORD *)(a1 + 48) = *(_DWORD *)(v11[8] + 104);
  v13 = *(_DWORD *)(v11[8] + 4 * v12 + 60);
  *(_DWORD *)(a1 + 60) = v13;
  *(_DWORD *)(a1 + 52) = *(_DWORD *)(v11[8] + 112);
  v14 = *(_DWORD *)(a1 + 44);
  *(_DWORD *)(a1 + 56) = *(_DWORD *)(v11[8] + 116);
  v15 = v14;
  *(_DWORD *)(a1 + 40) = v14;
  if ( v14 == 0 )
    return 0;
  v16 = *(_DWORD *)(a1 + 48);
  if ( v16 == 0 || v13 == 0 )
    return 0;
  v17 = *(_DWORD *)(a1 + 12);
  *(_DWORD *)(a1 + 64) = (int)*(unsigned __int16 *)(v17 + 20) % v11[173];
  v18 = *(unsigned __int16 *)(v17 + 20) / (int)v11[173];
  *(_DWORD *)(a1 + 8) = a4;
  *(_DWORD *)(a1 + 68) = v18;
  *(_BYTE *)(a1 + 128) = 15;
  *(_DWORD *)(a1 + 32) = 32 * *(_DWORD *)(v11[66] + 4 * v12 - 4);
  v19 = 32 * *(_DWORD *)(v11[66] + 4 * v12 - 4);
  *(_DWORD *)(a1 + 28) = *(_DWORD *)(a1 + 32);
  *(_DWORD *)(a1 + 36) = v19;
  *(_DWORD *)(a1 + 104) = 3;
  *(_DWORD *)(a1 + 96) = 4;
  *(_DWORD *)(a1 + 100) = 4;
  *(_DWORD *)(a1 + 80) = *(_DWORD *)(v15 + 4) >> 2;
  *(_DWORD *)(a1 + 84) = *(_DWORD *)(v15 + 8) >> 2;
  *(_DWORD *)(a1 + 88) = *(_DWORD *)(v16 + 4) >> 2;
  v20 = *(_DWORD *)(v16 + 8);
  *(_DWORD *)(a1 + 124) = a3;
  *(_DWORD *)(a1 + 72) = *(_DWORD *)(a1 + 80);
  *(_DWORD *)(a1 + 92) = v20 / 3;
  *(_DWORD *)(a1 + 76) = *(_DWORD *)(a1 + 84);
  sub_529D4(a1 + 188, *(char **)(a1 + 188), *(_BYTE **)(a1 + 192));
  v21 = a1;
  *(_BYTE *)(a1 + 129) = 0;
  *(_BYTE *)(a1 + 130) = -1;
  sub_11558(*(_DWORD *)(a1 + 24) + 64, &v23, *(_DWORD *)(*(_DWORD *)(a1 + 24) + 64), &v21);
  sub_3958C((_DWORD **)(*(_DWORD *)(a1 + 24) + 64));
  return 1;
}


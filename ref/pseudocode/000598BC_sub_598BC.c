/* ea:   0x000598BC */
/* name: sub_598BC */
/* size: 736 */
int __fastcall sub_598BC(int a1, int a2)
{
  struct CRITICAL_SECTION *v4; // r10
  unsigned int v5; // r2
  unsigned int v6; // r9
  int v7; // r1
  int v9; // r2
  int v10; // r8
  unsigned int v11; // r3
  int v12; // r0
  int v13; // r7
  int v14; // r6
  unsigned int v15; // r4
  _DWORD *v16; // r5
  int v17; // r0
  int v18; // r2
  int v19; // r0
  unsigned __int8 *v20; // r2
  int v21; // r0
  unsigned int v22; // r1
  void (__fastcall **v23)(int, _DWORD); // r2
  unsigned int v24; // r3
  _DWORD *v25; // r2
  unsigned int v26; // r0
  unsigned int v27; // r1
  void (__fastcall **v28)(int, _DWORD); // r2

  if ( *(_BYTE *)(a1 + 68) == 0 || *(_DWORD *)(a2 + 4) == 0 )
    return -1;
  v4 = (struct CRITICAL_SECTION *)(a1 + 40);
  EnterCriticalSection((LPCRITICAL_SECTION)(a1 + 40));
  v5 = *(_DWORD *)(a1 + 76);
  v6 = 0;
  if ( v5 != 0 )
  {
    v7 = *(_DWORD *)(a1 + 80);
    do
    {
      if ( *(_BYTE *)(v7 + 22) == 1 && *(_BYTE *)(v7 + 21) == 0 )
        break;
      ++v6;
      v7 += 1304;
    }
    while ( v6 < v5 );
  }
  if ( v6 == v5 )
  {
    LeaveCriticalSection((LPCRITICAL_SECTION)(a1 + 40));
    return -1;
  }
  v9 = *(_DWORD *)(a2 + 4);
  v10 = *(_DWORD *)(a1 + 80) + 1304 * v6;
  *(_DWORD *)(v10 + 40) = v9;
  *(_DWORD *)(v10 + 44) = v9 + 1084;
  v11 = sub_5987C(v9);
  v12 = *(_DWORD *)(v10 + 44);
  v13 = 0;
  *(_DWORD *)(v10 + 912) = 0;
  *(_DWORD *)(v10 + 784) = 0;
  v14 = v12 + (v11 << 10);
  v15 = 30;
  v16 = (_DWORD *)(v10 + 4);
  do
  {
    if ( *(_WORD *)(v15 + *(_DWORD *)(v10 + 40) + 12) != 0 )
    {
      v16[68] = (unsigned __int16)sub_58FC0(*(_WORD *)(v15 + *(_DWORD *)(v10 + 40) + 12)) << 17;
      v16[100] = (unsigned __int16)sub_58FC0(*(_WORD *)(v15 + *(_DWORD *)(v10 + 40) + 16)) << 17;
      if ( (unsigned __int16)sub_58FC0(*(_WORD *)(v15 + *(_DWORD *)(v10 + 40) + 18)) > 1u )
        v17 = (unsigned __int16)sub_58FC0(*(_WORD *)(v15 + *(_DWORD *)(v10 + 40) + 18)) << 17;
      else
        v17 = 0;
      v16[132] = v17;
      v16[164] = *(unsigned __int8 *)(v15 + *(_DWORD *)(v10 + 40) + 15);
      v18 = *(unsigned __int8 *)(v15 + *(_DWORD *)(v10 + 40) + 14);
      v16[228] = v13 + v14;
      v16[196] = v18;
      v13 += 2 * (unsigned __int16)sub_58FC0(*(_WORD *)(v15 + *(_DWORD *)(v10 + 40) + 12));
    }
    else
    {
      v16[228] = 0;
    }
    v15 += 30;
    ++v16;
  }
  while ( v15 < 0x3C0 );
  *(_DWORD *)(v10 + 24) = 44100;
  v19 = *(_DWORD *)(a2 + 20);
  *(_DWORD *)(v10 + 1300) = 50;
  *(_DWORD *)(v10 + 28) = v19;
  *(_BYTE *)(v10 + 32) = 1;
  *(_DWORD *)(v10 + 68) = 0;
  sub_5AEE8(v10, *(_DWORD *)(a2 + 24));
  v20 = *(unsigned __int8 **)(v10 + 40);
  *(_DWORD *)(v10 + 56) = 0;
  *(_DWORD *)(v10 + 72) = 6;
  *(_DWORD *)(v10 + 76) = 6;
  v21 = v20[952];
  *(_DWORD *)(v10 + 52) = 0;
  *(_DWORD *)(v10 + 48) = v21;
  v22 = v20[951];
  if ( v22 == 127 || v20[950] <= v22 )
    *(_DWORD *)(v10 + 60) = 0;
  else
    *(_DWORD *)(v10 + 60) = v22;
  v23 = *(void (__fastcall ***)(int, _DWORD))v10;
  *(_DWORD *)(v10 + 1268) = 0xFFFFFF;
  *(_DWORD *)(v10 + 1264) = 0xFFFFFF;
  *(_DWORD *)(v10 + 1272) = 0;
  v23[1](v10, *(_DWORD *)(a2 + 16));
  v24 = 0;
  v25 = (_DWORD *)v10;
  do
  {
    v26 = v24 + v10;
    v25[28] = 0;
    v27 = v24 + v10;
    v25[24] = 0;
    ++v24;
    v25[292] = 0;
    v25[321] = 0;
    *(_BYTE *)(v26 + 1276) = 0;
    ++v25;
    *(_BYTE *)(v27 + 1280) = 0;
  }
  while ( v24 < 4 );
  v28 = *(void (__fastcall ***)(int, _DWORD))v10;
  *(_BYTE *)(v10 + 20) = *(_BYTE *)(a2 + 12);
  (*v28)(v10, *(_DWORD *)(a2 + 8));
  sub_5AE70(v10);
  LeaveCriticalSection(v4);
  return v6;
}


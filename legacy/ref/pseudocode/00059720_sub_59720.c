/* ea:   0x00059720 */
/* name: sub_59720 */
/* size: 348 */
int __fastcall sub_59720(int a1, int a2)
{
  unsigned int v4; // r4
  int v5; // r8
  int v6; // r5
  unsigned int v7; // r3
  int v8; // r2
  int v9; // r0
  int v11; // r4
  int v12; // r0
  void (__fastcall **v13)(int, _DWORD); // r2
  void (__fastcall *v14)(int, _DWORD); // r3

  if ( *(_BYTE *)(a1 + 68) == 0 || *(_DWORD *)(a2 + 4) == 0 )
    return -1;
  EnterCriticalSection((LPCRITICAL_SECTION)(a1 + 40));
  v4 = *(_DWORD *)(a1 + 84);
  v5 = -1;
  v6 = 0;
  v7 = 0;
  if ( v4 != 0 )
  {
    v8 = *(_DWORD *)(a1 + 88);
    do
    {
      if ( *(_BYTE *)(v8 + 22) != 1 || *(_BYTE *)(v8 + 21) != 0 )
      {
        if ( *(_DWORD *)(v8 + 40) == *(_DWORD *)(a2 + 4) )
          ++v6;
      }
      else
      {
        v5 = v7;
        if ( *(_DWORD *)(a2 + 36) == -1 )
          goto LABEL_14;
      }
      ++v7;
      v8 += 60;
    }
    while ( v7 < v4 );
  }
  v9 = *(_DWORD *)(a2 + 36);
  if ( v9 != -1 && v6 >= v9 )
  {
LABEL_15:
    LeaveCriticalSection((LPCRITICAL_SECTION)(a1 + 40));
    return -1;
  }
LABEL_14:
  if ( v5 == -1 )
    goto LABEL_15;
  v11 = *(_DWORD *)(a1 + 88) + 60 * v5;
  *(_DWORD *)(v11 + 40) = *(_DWORD *)(a2 + 4);
  *(_DWORD *)(v11 + 24) = *(_DWORD *)(a2 + 12);
  v12 = *(_DWORD *)(a2 + 104);
  *(_BYTE *)(v11 + 32) = 1;
  *(_DWORD *)(v11 + 28) = v12;
  sub_5AEE8(v11, *(_DWORD *)(a2 + 108));
  *(_DWORD *)(v11 + 48) = 0;
  v13 = *(void (__fastcall ***)(int, _DWORD))v11;
  *(_DWORD *)(v11 + 52) = *(_DWORD *)(a2 + 16) << 8;
  v14 = *v13;
  *(_BYTE *)(v11 + 20) = *(_BYTE *)(a2 + 28);
  *(_DWORD *)(v11 + 16) = *(_DWORD *)(a2 + 32);
  *(_BYTE *)(v11 + 56) = *(_BYTE *)(a2 + 8);
  v14(v11, *(_DWORD *)(a2 + 24));
  sub_5AE70(v11);
  LeaveCriticalSection((LPCRITICAL_SECTION)(a1 + 40));
  return *(_DWORD *)(a1 + 76) + v5;
}


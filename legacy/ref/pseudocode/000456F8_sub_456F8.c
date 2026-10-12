/* ea:   0x000456F8 */
/* name: sub_456F8 */
/* size: 184 */
int __fastcall sub_456F8(int a1, int a2, int a3)
{
  _DWORD *v5; // r4
  _DWORD *v6; // r0

  if ( a1 == 0 )
    return -2147024809;
  *(_DWORD *)(a1 + 4) = a3;
  *(_DWORD *)(a1 + 8) = a3;
  *(_DWORD *)(a1 + 12) = a3;
  *(_DWORD *)(a1 + 16) = a2;
  *(_DWORD *)(a1 + 20) = 0;
  *(_DWORD *)(a1 + 24) = 0;
  InitializeCriticalSection((LPCRITICAL_SECTION)(a1 + 28));
  InitializeCriticalSection((LPCRITICAL_SECTION)(a1 + 48));
  InitializeCriticalSection((LPCRITICAL_SECTION)(a1 + 68));
  *(_DWORD *)(a1 + 92) = 0;
  *(_BYTE *)(a1 + 108) = 1;
  *(_DWORD *)(a1 + 116) = 0;
  *(_DWORD *)(a1 + 120) = 0;
  *(_DWORD *)(a1 + 124) = 0;
  *(_DWORD *)(a1 + 128) = 0;
  v5 = *(_DWORD **)(a1 + 16);
  if ( v5 != nullptr && *v5 != 0 )
  {
    do
    {
      ((void (__fastcall *)(int))v5[8])(1);
      v6 = v5 + 7;
      if ( *(_DWORD *)a1 != 88 )
        v6 = v5 + 9;
      v5 = v6;
    }
    while ( *v6 != 0 );
  }
  return 0;
}


/* ea:   0x000457B0 */
/* name: sub_457B0 */
/* size: 356 */
int __fastcall sub_457B0(int a1)
{
  int i; // r4
  _DWORD *v4; // r4
  void (__fastcall *v5)(_DWORD); // r1
  _DWORD *v6; // r0
  _DWORD *v7; // r5
  _DWORD *v8; // r4
  unsigned int v9; // r4

  if ( a1 == 0 )
    return -2147024809;
  for ( i = 0; i < *(_DWORD *)(a1 + 100); ++i )
    UnregisterClassW((LPCWSTR)*(unsigned __int16 *)(*(_DWORD *)(a1 + 96) + 2 * i), *(HINSTANCE *)(a1 + 4));
  sub_45580(a1 + 96);
  v4 = *(_DWORD **)(a1 + 16);
  if ( v4 != nullptr && *v4 != 0 )
  {
    do
    {
      if ( v4[4] != 0 )
        (*(void (__fastcall **)(_DWORD))(*(_DWORD *)v4[4] + 8))(v4[4]);
      v5 = (void (__fastcall *)(_DWORD))v4[8];
      v4[4] = 0;
      v5(0);
      v6 = v4 + 7;
      if ( *(_DWORD *)a1 != 88 )
        v6 = v4 + 9;
      v4 = v6;
    }
    while ( *v6 != 0 );
  }
  DeleteCriticalSection((LPCRITICAL_SECTION)(a1 + 28));
  DeleteCriticalSection((LPCRITICAL_SECTION)(a1 + 48));
  DeleteCriticalSection((LPCRITICAL_SECTION)(a1 + 68));
  v7 = *(_DWORD **)(a1 + 128);
  if ( v7 != nullptr )
  {
    do
    {
      ((void (__fastcall *)(_DWORD))*v7)(v7[1]);
      v8 = (_DWORD *)v7[2];
      operator delete(v7);
      v7 = v8;
    }
    while ( v8 != nullptr );
  }
  if ( *(_DWORD *)(a1 + 24) != 0 && *(_BYTE *)(a1 + 108) != 0 )
  {
    if ( *(_DWORD *)(a1 + 120) != 0 )
    {
      v9 = 0;
      do
        HeapDestroy(*(HANDLE *)(*(_DWORD *)(a1 + 120) + 4 * v9++));
      while ( v9 <= *(_DWORD *)(a1 + 116) );
    }
    HeapDestroy(*(HANDLE *)(a1 + 24));
  }
  return 0;
}


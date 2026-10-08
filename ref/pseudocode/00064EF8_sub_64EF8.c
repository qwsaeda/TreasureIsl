/* ea:   0x00064EF8 */
/* name: sub_64EF8 */
/* size: 316 */
int __fastcall sub_64EF8(int a1, int a2, int a3)
{
  void *v6; // r0
  unsigned int v7; // r0
  void *v8; // r0
  void *v9; // r0
  int v11; // r0

  if ( a3 == 0 || a2 == 0 )
    __debugbreak();
  if ( *(_BYTE *)(a1 + 12) != 0 )
  {
    v6 = *(void **)(a1 + 16);
    if ( v6 != nullptr )
    {
      operator delete(v6);
      *(_DWORD *)(a1 + 16) = 0;
    }
    *(_BYTE *)(a1 + 12) = 0;
  }
  if ( (*(unsigned __int8 (__fastcall **)(int, int))(*(_DWORD *)a2 + 32))(a2, a3) != 0 )
  {
    v7 = (*(int (__fastcall **)(int, int))(*(_DWORD *)a2 + 20))(a2, a3);
    v8 = operator new(v7);
    *(_DWORD *)(a1 + 16) = v8;
    if ( v8 == nullptr )
      return 0;
    if ( (*(unsigned __int8 (__fastcall **)(int, void *, int, int))(*(_DWORD *)a2 + 36))(a2, v8, a3, 1) == 0 )
    {
      v9 = *(void **)(a1 + 16);
      if ( v9 != nullptr )
      {
        operator delete(v9);
        *(_DWORD *)(a1 + 16) = 0;
      }
      return 0;
    }
    *(_DWORD *)(a1 + 4) = a3;
    *(_DWORD *)(a1 + 8) = a2;
    *(_DWORD *)(a1 + 24) = 0;
    v11 = (*(int (__fastcall **)(int, int))(*(_DWORD *)a2 + 20))(a2, a3);
    *(_BYTE *)(a1 + 12) = 1;
    *(_DWORD *)(a1 + 20) = v11;
    return 1;
  }
  else
  {
    *(_DWORD *)(a1 + 24) = 0;
    *(_DWORD *)(a1 + 4) = a3;
    *(_DWORD *)(a1 + 8) = a2;
    *(_BYTE *)(a1 + 12) = 0;
    *(_DWORD *)(a1 + 16) = (*(int (__fastcall **)(int, int))(*(_DWORD *)a2 + 40))(a2, a3);
    return 1;
  }
}


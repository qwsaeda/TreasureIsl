/* ea:   0x00062298 */
/* name: sub_62298 */
/* size: 320 */
int __fastcall sub_62298(int a1, char a2, int a3)
{
  int v7; // r1
  void (__fastcall ***v8)(_DWORD, int); // [sp+0h] [bp-2Ch] BYREF
  void (__fastcall ***v9)(_DWORD, int); // [sp+4h] [bp-28h] BYREF
  _DWORD v10[9]; // [sp+8h] [bp-24h] BYREF

  if ( a3 == 0 )
    __debugbreak();
  if ( *(_BYTE *)(a1 + 4) != 0 )
    (*(void (__fastcall **)(int))(*(_DWORD *)a1 + 8))(a1);
  if ( a2 == 2 )
  {
    v8 = (void (__fastcall ***)(_DWORD, int))sub_637C8(a3);
    sub_62428(a1 + 8, &v8);
    if ( v8 != nullptr )
      (**v8)(v8, 1);
  }
  else if ( a2 == 1 )
  {
    v9 = (void (__fastcall ***)(_DWORD, int))sub_632F8(a3);
    sub_62428(a1 + 8, &v9);
    if ( v9 != nullptr )
      (**v9)(v9, 1);
  }
  if ( *(_DWORD *)(a1 + 8) == 0 )
    return 0;
  (*(void (__fastcall **)(_DWORD, _DWORD *))(**(_DWORD **)(a1 + 8) + 28))(*(_DWORD *)(a1 + 8), v10);
  *(_DWORD *)(a1 + 12) = 0;
  v7 = v10[1];
  *(_DWORD *)(a1 + 16) = v10[0];
  *(_DWORD *)(a1 + 20) = 0;
  *(_DWORD *)(a1 + 24) = v7;
  word_8122C = -2017;
  *(_BYTE *)(a1 + 4) = 1;
  return 1;
}


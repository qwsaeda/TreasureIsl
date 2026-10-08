/* ea:   0x000407B8 */
/* name: sub_407B8 */
/* size: 164 */
int __fastcall sub_407B8(int a1)
{
  int v2; // r1
  void (__fastcall ***v3)(_DWORD, int); // r3
  void (__fastcall ***v4)(_DWORD, int); // r3

  v2 = *(_DWORD *)(a1 + 48);
  if ( v2 != 0 )
  {
    (*(void (__fastcall **)(_DWORD))(*(_DWORD *)v2 + 8))(*(_DWORD *)(a1 + 48));
    v3 = *(void (__fastcall ****)(_DWORD, int))(a1 + 48);
    if ( v3 != nullptr )
      (**v3)(*(_DWORD *)(a1 + 48), 1);
    *(_DWORD *)(a1 + 48) = 0;
  }
  if ( *(_DWORD *)(a1 + 92) != 0 )
  {
    (*(void (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 92) + 8))(*(_DWORD *)(a1 + 92));
    v4 = *(void (__fastcall ****)(_DWORD, int))(a1 + 92);
    if ( v4 != nullptr )
      (**v4)(*(_DWORD *)(a1 + 92), 1);
    *(_DWORD *)(a1 + 92) = 0;
  }
  return 1;
}


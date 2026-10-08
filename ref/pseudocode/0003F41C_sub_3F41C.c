/* ea:   0x0003F41C */
/* name: sub_3F41C */
/* size: 200 */
int __fastcall sub_3F41C(int a1)
{
  int v2; // r1
  void (__fastcall ***v3)(_DWORD, int); // r3
  void (__fastcall ***v4)(_DWORD, int); // r3
  void **i; // r4

  v2 = *(_DWORD *)(a1 + 48);
  if ( v2 != 0 )
  {
    (*(void (__fastcall **)(_DWORD))(*(_DWORD *)v2 + 8))(*(_DWORD *)(a1 + 48));
    v3 = *(void (__fastcall ****)(_DWORD, int))(a1 + 48);
    if ( v3 != nullptr )
      (**v3)(*(_DWORD *)(a1 + 48), 1);
    *(_DWORD *)(a1 + 48) = 0;
  }
  if ( *(_DWORD *)(a1 + 72) != 0 )
  {
    (*(void (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 72) + 8))(*(_DWORD *)(a1 + 72));
    v4 = *(void (__fastcall ****)(_DWORD, int))(a1 + 72);
    if ( v4 != nullptr )
      (**v4)(*(_DWORD *)(a1 + 72), 1);
    *(_DWORD *)(a1 + 72) = 0;
  }
  for ( i = *(void ***)(a1 + 32); i != *(void ***)(a1 + 36); ++i )
  {
    if ( *i != nullptr )
      free(*i);
  }
  return 1;
}


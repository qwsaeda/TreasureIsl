/* ea:   0x00042A34 */
/* name: sub_42A34 */
/* size: 180 */
int __fastcall sub_42A34(int a1)
{
  void *v2; // r0
  void (__fastcall ***v3)(_DWORD, int); // r3
  int result; // r0
  int (__fastcall ***v5)(_DWORD, int); // r3

  v2 = *(void **)(a1 + 12);
  if ( v2 != nullptr )
  {
    free(v2);
    *(_DWORD *)(a1 + 12) = 0;
  }
  if ( *(_DWORD *)(a1 + 16) != 0 )
  {
    (*(void (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 16) + 8))(*(_DWORD *)(a1 + 16));
    v3 = *(void (__fastcall ****)(_DWORD, int))(a1 + 16);
    if ( v3 != nullptr )
      (**v3)(*(_DWORD *)(a1 + 16), 1);
    *(_DWORD *)(a1 + 16) = 0;
  }
  result = *(_DWORD *)(a1 + 20);
  if ( result != 0 )
  {
    result = (*(int (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 20) + 8))(*(_DWORD *)(a1 + 20));
    v5 = *(int (__fastcall ****)(_DWORD, int))(a1 + 20);
    if ( v5 != nullptr )
      result = (**v5)(*(_DWORD *)(a1 + 20), 1);
    *(_DWORD *)(a1 + 20) = 0;
  }
  return result;
}


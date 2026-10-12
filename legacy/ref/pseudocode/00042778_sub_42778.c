/* ea:   0x00042778 */
/* name: sub_42778 */
/* size: 112 */
int __fastcall sub_42778(int a1)
{
  void *v2; // r0
  void (__fastcall ***v3)(_DWORD, int); // r3

  v2 = *(void **)(a1 + 52);
  if ( v2 != nullptr )
  {
    operator delete(v2);
    *(_DWORD *)(a1 + 52) = 0;
  }
  if ( *(_DWORD *)(a1 + 48) != 0 )
  {
    (*(void (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 48) + 8))(*(_DWORD *)(a1 + 48));
    v3 = *(void (__fastcall ****)(_DWORD, int))(a1 + 48);
    if ( v3 != nullptr )
      (**v3)(*(_DWORD *)(a1 + 48), 1);
    *(_DWORD *)(a1 + 48) = 0;
  }
  return 1;
}


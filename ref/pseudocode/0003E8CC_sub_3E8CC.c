/* ea:   0x0003E8CC */
/* name: sub_3E8CC */
/* size: 116 */
void __fastcall sub_3E8CC(int a1)
{
  int v2; // r4
  char *v3; // r4

  if ( *(_DWORD *)(a1 + 12) != 0 )
  {
    sub_59B9C(*(_DWORD *)(a1 + 20));
    v2 = *(_DWORD *)(a1 + 12);
    if ( *(_DWORD *)(v2 + 4) != 0 )
    {
      (*(void (__fastcall **)(_DWORD, _DWORD))(**(_DWORD **)(a1 + 16) + 12))(*(_DWORD *)(a1 + 16), *(_DWORD *)(v2 + 4));
      *(_DWORD *)(v2 + 4) = 0;
    }
    v3 = *(char **)(a1 + 12);
    if ( v3 != nullptr )
    {
      sub_58784(v3 + 8);
      operator delete(v3);
    }
    *(_DWORD *)(a1 + 12) = 0;
  }
}


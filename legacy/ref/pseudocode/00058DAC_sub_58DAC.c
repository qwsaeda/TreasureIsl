/* ea:   0x00058DAC */
/* name: sub_58DAC */
/* size: 52 */
void __fastcall sub_58DAC(int a1)
{
  void *v2; // r0

  if ( *(_BYTE *)(a1 + 112) != 0 )
  {
    v2 = *(void **)(a1 + 4);
    if ( v2 != nullptr )
    {
      operator delete(v2);
      *(_DWORD *)(a1 + 4) = 0;
      *(_BYTE *)(a1 + 112) = 0;
    }
  }
}


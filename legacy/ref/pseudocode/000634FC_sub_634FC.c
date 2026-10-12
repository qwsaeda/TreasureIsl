/* ea:   0x000634FC */
/* name: sub_634FC */
/* size: 72 */
int __fastcall sub_634FC(int a1)
{
  void *v2; // r0
  int result; // r0

  v2 = *(void **)(a1 + 8);
  if ( v2 != nullptr )
  {
    free(v2);
    *(_DWORD *)(a1 + 8) = 0;
  }
  result = *(unsigned __int8 *)(a1 + 4);
  if ( *(_BYTE *)(a1 + 4) != 0 )
  {
    result = GXCloseDisplay();
    *(_BYTE *)(a1 + 4) = 0;
  }
  return result;
}


/* ea:   0x000583DC */
/* name: sub_583DC */
/* size: 24 */
int __fastcall sub_583DC(int a1)
{
  int result; // r0

  result = *(_DWORD *)(a1 + 1024);
  if ( result != 0 )
  {
    *(_DWORD *)(a1 + 1008) = 1;
    return 1;
  }
  return result;
}


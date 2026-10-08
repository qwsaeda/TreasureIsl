/* ea:   0x00033CC4 */
/* name: sub_33CC4 */
/* size: 152 */
int __fastcall sub_33CC4(int a1)
{
  _DWORD *i; // r5
  void *v3; // r0
  void *v4; // r0

  for ( i = *(_DWORD **)(*(_DWORD *)(a1 + 4) + 8); i != *(_DWORD **)(a1 + 4); i = sub_16A30(i) )
  {
    v3 = (void *)i[5];
    if ( v3 != nullptr )
    {
      free(v3);
      i[5] = 0;
    }
  }
  sub_332DC((_DWORD *)(a1 + 4));
  if ( *(_DWORD *)(a1 + 44) != 0 )
  {
    free(*(void **)(a1 + 44));
    *(_DWORD *)(a1 + 44) = 0;
  }
  v4 = *(void **)(a1 + 40);
  if ( v4 != nullptr )
  {
    free(v4);
    *(_DWORD *)(a1 + 40) = 0;
  }
  return 1;
}


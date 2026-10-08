/* ea:   0x000421C0 */
/* name: sub_421C0 */
/* size: 76 */
int __fastcall sub_421C0(_DWORD *a1)
{
  void **i; // r4
  void *v3; // r0

  for ( i = (void **)a1[8]; i != (void **)a1[9]; ++i )
  {
    if ( *i != nullptr )
      free(*i);
  }
  v3 = (void *)a1[13];
  if ( v3 != nullptr )
  {
    free(v3);
    a1[13] = 0;
  }
  return 1;
}


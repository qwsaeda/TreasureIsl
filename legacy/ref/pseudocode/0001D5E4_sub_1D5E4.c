/* ea:   0x0001D5E4 */
/* name: sub_1D5E4 */
/* size: 76 */
int __fastcall sub_1D5E4(_DWORD *a1)
{
  void **i; // r4
  void *v3; // r0

  for ( i = (void **)a1[8]; i != (void **)a1[9]; ++i )
  {
    if ( *i != nullptr )
      free(*i);
  }
  v3 = (void *)a1[4];
  if ( v3 != nullptr )
  {
    free(v3);
    a1[4] = 0;
  }
  return 1;
}


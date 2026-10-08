/* ea:   0x00050080 */
/* name: sub_50080 */
/* size: 52 */
int __fastcall sub_50080(int a1)
{
  void **i; // r5

  for ( i = *(void ***)(a1 + 32); i != *(void ***)(a1 + 36); ++i )
  {
    if ( *i != nullptr )
      free(*i);
  }
  return 1;
}


/* ea:   0x0004ED28 */
/* name: sub_4ED28 */
/* size: 20 */
void __fastcall sub_4ED28(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}


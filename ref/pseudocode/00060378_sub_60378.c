/* ea:   0x00060378 */
/* name: sub_60378 */
/* size: 20 */
void __fastcall sub_60378(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}


/* ea:   0x00050218 */
/* name: sub_50218 */
/* size: 20 */
void __fastcall sub_50218(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

